use super::{find_matching_pods, pod_pattern_regex, should_show_line, PodInfo};
use ansi_to_tui::IntoText;
use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    MouseButton, MouseEventKind,
};
use crossterm::ExecutableCommand;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    DefaultTerminal,
};
use regex::Regex;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

const MAX_LOG_LINES: usize = 5000;

const TUI_COLORS: &[Color] = &[
    Color::Cyan,
    Color::Green,
    Color::Magenta,
    Color::Yellow,
    Color::Blue,
    Color::LightCyan,
    Color::LightGreen,
    Color::LightMagenta,
];

enum TrackEvent {
    LogLine { pod_key: String, text: String },
    NewPod { pod: PodInfo, alive: Arc<AtomicBool> },
}

struct PodPane {
    key: String,
    color: Color,
    lines: VecDeque<String>,
    alive: Arc<AtomicBool>,
    scroll_up: Option<usize>,
}

impl PodPane {
    fn new(key: String, color: Color, alive: Arc<AtomicBool>) -> Self {
        Self {
            key,
            color,
            lines: VecDeque::with_capacity(MAX_LOG_LINES),
            alive,
            scroll_up: None,
        }
    }

    fn push_line(&mut self, line: String) {
        if self.lines.len() >= MAX_LOG_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    fn scroll_offset(&self, inner_height: usize) -> u16 {
        let auto = self.lines.len().saturating_sub(inner_height);
        match self.scroll_up {
            None => auto as u16,
            Some(up) => auto.saturating_sub(up) as u16,
        }
    }

    fn is_following(&self) -> bool {
        self.scroll_up.is_none()
    }
}

pub fn run(pods: Vec<PodInfo>, regexes: Vec<Regex>, err_only: bool) -> Result<()> {
    let mut terminal = ratatui::init();
    std::io::stdout().execute(EnableMouseCapture)?;
    let result = run_tui(&mut terminal, pods, regexes, err_only);
    let _ = std::io::stdout().execute(DisableMouseCapture);
    ratatui::restore();
    result
}

fn run_tui(
    terminal: &mut DefaultTerminal,
    pods: Vec<PodInfo>,
    initial_regexes: Vec<Regex>,
    err_only: bool,
) -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let (tx, rx) = mpsc::channel::<TrackEvent>();
    let active_pods: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let closed_pods: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let regexes: Arc<Mutex<Vec<Regex>>> = Arc::new(Mutex::new(initial_regexes));
    let color_counter = Arc::new(AtomicUsize::new(0));

    let mut panes: Vec<PodPane> = Vec::new();
    let mut pane_index: HashMap<String, usize> = HashMap::new();
    let mut selected: usize = 0;
    let mut expanded = false;
    let mut input_mode = false;
    let mut input_buffer = String::new();
    let mut pane_rects: Vec<(usize, Rect)> = vec![];

    for pod in &pods {
        let key = pod.key();
        let cidx = color_counter.fetch_add(1, Ordering::SeqCst);
        let color = TUI_COLORS[cidx % TUI_COLORS.len()];
        let alive = Arc::new(AtomicBool::new(true));
        pane_index.insert(key.clone(), panes.len());
        panes.push(PodPane::new(key.clone(), color, alive.clone()));
        active_pods.lock().unwrap().insert(key);
        spawn_tui_log_follower(
            &pod.namespace,
            &pod.name,
            err_only,
            running.clone(),
            alive,
            active_pods.clone(),
            tx.clone(),
        );
    }

    {
        let poll_running = running.clone();
        let poll_tx = tx.clone();
        let poll_active = active_pods.clone();
        let poll_closed = closed_pods.clone();
        let poll_regexes = regexes.clone();
        thread::spawn(move || {
            while poll_running.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_secs(5));
                if !poll_running.load(Ordering::SeqCst) {
                    break;
                }
                let current_regexes = poll_regexes.lock().unwrap().clone();
                if let Ok(new_pods) = find_matching_pods(&current_regexes) {
                    for pod in new_pods {
                        let key = pod.key();
                        if poll_closed.lock().unwrap().contains(&key) {
                            continue;
                        }
                        let should_spawn = {
                            let mut active = poll_active.lock().unwrap();
                            if active.contains(&key) {
                                false
                            } else {
                                active.insert(key);
                                true
                            }
                        };
                        if should_spawn {
                            let alive = Arc::new(AtomicBool::new(true));
                            spawn_tui_log_follower(
                                &pod.namespace,
                                &pod.name,
                                err_only,
                                poll_running.clone(),
                                alive.clone(),
                                poll_active.clone(),
                                poll_tx.clone(),
                            );
                            let _ = poll_tx.send(TrackEvent::NewPod { pod, alive });
                        }
                    }
                }
            }
        });
    }

    loop {
        while let Ok(evt) = rx.try_recv() {
            match evt {
                TrackEvent::LogLine { pod_key, text } => {
                    if let Some(&idx) = pane_index.get(&pod_key) {
                        panes[idx].push_line(text);
                    }
                }
                TrackEvent::NewPod { pod, alive } => {
                    let key = pod.key();
                    if !pane_index.contains_key(&key) {
                        let cidx = color_counter.fetch_add(1, Ordering::SeqCst);
                        let color = TUI_COLORS[cidx % TUI_COLORS.len()];
                        pane_index.insert(key.clone(), panes.len());
                        panes.push(PodPane::new(key, color, alive));
                    }
                }
            }
        }

        let term_size = terminal.size()?;
        {
            let main_layout =
                Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(term_size.into());
            let visible_indices: Vec<usize> = if expanded {
                vec![selected]
            } else {
                (0..panes.len()).collect()
            };
            let vis_count = visible_indices.len().max(1) as u32;
            let constraints: Vec<Constraint> = visible_indices
                .iter()
                .map(|_| Constraint::Ratio(1, vis_count))
                .collect();
            let chunks = Layout::vertical(constraints).split(main_layout[0]);
            pane_rects = visible_indices
                .iter()
                .zip(chunks.iter())
                .map(|(&i, r)| (i, *r))
                .collect();
        }

        terminal.draw(|frame| {
            let main_chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)])
                .split(frame.area());

            if !panes.is_empty() {
                let visible: Vec<(usize, &PodPane)> = if expanded {
                    panes
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i == selected)
                        .collect()
                } else {
                    panes.iter().enumerate().collect()
                };

                let vis_count = visible.len() as u32;
                let constraints: Vec<Constraint> = visible
                    .iter()
                    .map(|_| Constraint::Ratio(1, vis_count))
                    .collect();
                let chunks = Layout::vertical(constraints).split(main_chunks[0]);

                for (ci, (i, pane)) in visible.iter().enumerate() {
                    let is_selected = *i == selected;
                    let border_style = if is_selected {
                        Style::default()
                            .fg(pane.color)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(pane.color)
                            .add_modifier(Modifier::DIM)
                    };

                    let title = if pane.is_following() {
                        format!(" {} ", pane.key)
                    } else {
                        format!(" {} [PAUSED] ", pane.key)
                    };

                    let title_style = if !pane.is_following() {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(pane.color)
                            .add_modifier(Modifier::BOLD)
                    };

                    let block = Block::bordered()
                        .title(title)
                        .title_style(title_style)
                        .border_style(border_style);

                    let inner_height = chunks[ci].height.saturating_sub(2) as usize;
                    let scroll_offset = pane.scroll_offset(inner_height);

                    let content = pane.lines.iter().cloned().collect::<Vec<_>>().join("\n");
                    let text =
                        content.as_bytes().into_text().unwrap_or_else(|_| Text::raw(&content));
                    let paragraph = Paragraph::new(text)
                        .block(block)
                        .scroll((scroll_offset, 0));

                    frame.render_widget(paragraph, chunks[ci]);

                    if pane.lines.len() > inner_height {
                        let max_scroll = pane.lines.len().saturating_sub(inner_height);
                        let mut scrollbar_state = ScrollbarState::new(max_scroll)
                            .position(scroll_offset as usize);
                        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                            .style(Style::default().fg(if is_selected {
                                pane.color
                            } else {
                                Color::DarkGray
                            }));
                        frame.render_stateful_widget(
                            scrollbar,
                            chunks[ci].inner(Margin {
                                vertical: 1,
                                horizontal: 0,
                            }),
                            &mut scrollbar_state,
                        );
                    }
                }
            }

            let status_line = if input_mode {
                Line::from(vec![
                    Span::styled(
                        " Pattern: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(input_buffer.as_str()),
                    Span::styled("\u{2588}", Style::default().fg(Color::White)),
                    Span::raw("  "),
                    Span::styled(
                        "Enter",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": add  "),
                    Span::styled(
                        "Esc",
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": cancel"),
                ])
            } else {
                Line::from(vec![
                    Span::raw(" "),
                    Span::styled(
                        "Tab",
                        Style::default()
                            .fg(Color::LightCyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": switch  ", Style::default().fg(Color::White)),
                    Span::styled(
                        "\u{2191}\u{2193}",
                        Style::default()
                            .fg(Color::LightCyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": scroll  ", Style::default().fg(Color::White)),
                    Span::styled(
                        "End",
                        Style::default()
                            .fg(Color::LightCyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": follow  ", Style::default().fg(Color::White)),
                    Span::styled(
                        "f",
                        Style::default()
                            .fg(Color::LightCyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        if expanded { ": collapse  " } else { ": expand  " },
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        "a",
                        Style::default()
                            .fg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": add pod  ", Style::default().fg(Color::White)),
                    Span::styled(
                        "d",
                        Style::default()
                            .fg(Color::LightRed)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": close  ", Style::default().fg(Color::White)),
                    Span::styled(
                        "q",
                        Style::default()
                            .fg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": quit", Style::default().fg(Color::White)),
                ])
            };

            frame.render_widget(
                Paragraph::new(status_line).style(Style::default().bg(Color::Rgb(30, 30, 30))),
                main_chunks[1],
            );
        })?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    if input_mode {
                        handle_input_mode(
                            key_event.code,
                            &mut input_mode,
                            &mut input_buffer,
                            err_only,
                            &running,
                            &active_pods,
                            &closed_pods,
                            &regexes,
                            &tx,
                        );
                    } else {
                        let avail = term_size.height.saturating_sub(1);
                        let pane_h = if panes.is_empty() {
                            avail
                        } else {
                            avail / panes.len() as u16
                        };
                        let page_size = pane_h.saturating_sub(2) as usize;

                        let should_quit = handle_normal_mode(
                            key_event.code,
                            key_event.modifiers,
                            &mut selected,
                            &mut input_mode,
                            &mut expanded,
                            &mut panes,
                            &mut pane_index,
                            &running,
                            &closed_pods,
                            page_size,
                        );
                        if should_quit {
                            break;
                        }
                    }
                }
                Event::Mouse(mouse_event) if !input_mode => {
                    handle_mouse_event(
                        mouse_event,
                        &mut selected,
                        &mut panes,
                        &pane_rects,
                    );
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn handle_mouse_event(
    mouse: crossterm::event::MouseEvent,
    selected: &mut usize,
    panes: &mut Vec<PodPane>,
    pane_rects: &[(usize, Rect)],
) {
    let col = mouse.column;
    let row = mouse.row;

    let hit = pane_rects
        .iter()
        .find(|(_, rect)| {
            col >= rect.x
                && col < rect.x + rect.width
                && row >= rect.y
                && row < rect.y + rect.height
        })
        .copied();

    let Some((pane_idx, rect)) = hit else {
        return;
    };

    match mouse.kind {
        MouseEventKind::ScrollUp => {
            *selected = pane_idx;
            if let Some(pane) = panes.get_mut(pane_idx) {
                let max = pane.lines.len().saturating_sub(1);
                let current = pane.scroll_up.unwrap_or(0);
                pane.scroll_up = Some(current.saturating_add(3).min(max));
            }
        }
        MouseEventKind::ScrollDown => {
            *selected = pane_idx;
            if let Some(pane) = panes.get_mut(pane_idx) {
                if let Some(up) = pane.scroll_up {
                    pane.scroll_up = if up <= 3 { None } else { Some(up - 3) };
                }
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            *selected = pane_idx;
            scroll_to_scrollbar_pos(col, row, &rect, panes.get_mut(pane_idx));
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            scroll_to_scrollbar_pos(col, row, &rect, panes.get_mut(pane_idx));
        }
        _ => {}
    }
}

fn scroll_to_scrollbar_pos(col: u16, row: u16, rect: &Rect, pane: Option<&mut PodPane>) {
    let scrollbar_col = rect.x + rect.width - 1;
    if col < scrollbar_col.saturating_sub(1) {
        return;
    }
    let inner_top = rect.y + 1;
    let inner_bottom = rect.y + rect.height.saturating_sub(1);
    let inner_height = inner_bottom.saturating_sub(inner_top) as usize;
    if row >= inner_top && row < inner_bottom && inner_height > 0 {
        if let Some(pane) = pane {
            let click_pos = (row - inner_top) as usize;
            let max_scroll = pane.lines.len().saturating_sub(inner_height);
            let target = (click_pos * max_scroll) / inner_height.max(1);
            let scroll_up = max_scroll.saturating_sub(target);
            pane.scroll_up = if scroll_up == 0 { None } else { Some(scroll_up) };
        }
    }
}

fn handle_input_mode(
    code: KeyCode,
    input_mode: &mut bool,
    input_buffer: &mut String,
    err_only: bool,
    running: &Arc<AtomicBool>,
    active_pods: &Arc<Mutex<HashSet<String>>>,
    closed_pods: &Arc<Mutex<HashSet<String>>>,
    regexes: &Arc<Mutex<Vec<Regex>>>,
    tx: &mpsc::Sender<TrackEvent>,
) {
    match code {
        KeyCode::Enter => {
            if !input_buffer.is_empty() {
                let pattern = input_buffer.clone();
                input_buffer.clear();
                *input_mode = false;
                add_pattern(
                    &pattern, err_only, running, active_pods, closed_pods, regexes, tx,
                );
            }
        }
        KeyCode::Esc => {
            input_buffer.clear();
            *input_mode = false;
        }
        KeyCode::Backspace => {
            input_buffer.pop();
        }
        KeyCode::Char(c) => {
            input_buffer.push(c);
        }
        _ => {}
    }
}

fn handle_normal_mode(
    code: KeyCode,
    modifiers: KeyModifiers,
    selected: &mut usize,
    input_mode: &mut bool,
    expanded: &mut bool,
    panes: &mut Vec<PodPane>,
    pane_index: &mut HashMap<String, usize>,
    running: &Arc<AtomicBool>,
    closed_pods: &Arc<Mutex<HashSet<String>>>,
    page_size: usize,
) -> bool {
    match code {
        KeyCode::Char('q') => {
            running.store(false, Ordering::SeqCst);
            return true;
        }
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            running.store(false, Ordering::SeqCst);
            return true;
        }
        KeyCode::Char('f') => {
            *expanded = !*expanded;
        }
        KeyCode::Esc => {
            if *expanded {
                *expanded = false;
            }
        }
        KeyCode::Tab | KeyCode::Char('j') => {
            if !panes.is_empty() {
                *selected = (*selected + 1) % panes.len();
            }
        }
        KeyCode::BackTab | KeyCode::Char('k') => {
            if !panes.is_empty() {
                *selected = selected.checked_sub(1).unwrap_or(panes.len() - 1);
            }
        }
        KeyCode::Up => {
            if let Some(pane) = panes.get_mut(*selected) {
                let max = pane.lines.len().saturating_sub(1);
                let current = pane.scroll_up.unwrap_or(0);
                pane.scroll_up = Some(current.saturating_add(1).min(max));
            }
        }
        KeyCode::Down => {
            if let Some(pane) = panes.get_mut(*selected) {
                if let Some(up) = pane.scroll_up {
                    pane.scroll_up = if up <= 1 { None } else { Some(up - 1) };
                }
            }
        }
        KeyCode::PageUp => {
            if let Some(pane) = panes.get_mut(*selected) {
                let max = pane.lines.len().saturating_sub(1);
                let current = pane.scroll_up.unwrap_or(0);
                pane.scroll_up = Some(current.saturating_add(page_size).min(max));
            }
        }
        KeyCode::PageDown => {
            if let Some(pane) = panes.get_mut(*selected) {
                if let Some(up) = pane.scroll_up {
                    pane.scroll_up = if up <= page_size { None } else { Some(up - page_size) };
                }
            }
        }
        KeyCode::Home => {
            if let Some(pane) = panes.get_mut(*selected) {
                let max = pane.lines.len().saturating_sub(1);
                if max > 0 {
                    pane.scroll_up = Some(max);
                }
            }
        }
        KeyCode::End => {
            if let Some(pane) = panes.get_mut(*selected) {
                pane.scroll_up = None;
            }
        }
        KeyCode::Char('a') => {
            *input_mode = true;
        }
        KeyCode::Char('d') => {
            if !panes.is_empty() {
                let removed = panes.remove(*selected);
                removed.alive.store(false, Ordering::SeqCst);
                closed_pods.lock().unwrap().insert(removed.key.clone());
                *pane_index = rebuild_pane_index(panes);
                if panes.is_empty() {
                    *selected = 0;
                } else {
                    *selected = (*selected).min(panes.len() - 1);
                }
            }
        }
        _ => {}
    }
    false
}

fn add_pattern(
    pattern: &str,
    err_only: bool,
    running: &Arc<AtomicBool>,
    active_pods: &Arc<Mutex<HashSet<String>>>,
    closed_pods: &Arc<Mutex<HashSet<String>>>,
    regexes: &Arc<Mutex<Vec<Regex>>>,
    tx: &mpsc::Sender<TrackEvent>,
) {
    let new_regex = pod_pattern_regex(pattern);
    regexes.lock().unwrap().push(new_regex.clone());

    let disc_running = running.clone();
    let disc_active = active_pods.clone();
    let disc_closed = closed_pods.clone();
    let disc_tx = tx.clone();

    thread::spawn(move || {
        if let Ok(pods) = find_matching_pods(&[new_regex]) {
            for pod in pods {
                let key = pod.key();
                if disc_closed.lock().unwrap().contains(&key) {
                    continue;
                }
                let should_spawn = {
                    let mut active = disc_active.lock().unwrap();
                    if active.contains(&key) {
                        false
                    } else {
                        active.insert(key);
                        true
                    }
                };
                if should_spawn {
                    let alive = Arc::new(AtomicBool::new(true));
                    spawn_tui_log_follower(
                        &pod.namespace,
                        &pod.name,
                        err_only,
                        disc_running.clone(),
                        alive.clone(),
                        disc_active.clone(),
                        disc_tx.clone(),
                    );
                    let _ = disc_tx.send(TrackEvent::NewPod { pod, alive });
                }
            }
        }
    });
}

fn rebuild_pane_index(panes: &[PodPane]) -> HashMap<String, usize> {
    panes
        .iter()
        .enumerate()
        .map(|(i, p)| (p.key.clone(), i))
        .collect()
}

fn spawn_tui_log_follower(
    namespace: &str,
    pod_name: &str,
    err_only: bool,
    running: Arc<AtomicBool>,
    alive: Arc<AtomicBool>,
    active_pods: Arc<Mutex<HashSet<String>>>,
    tx: mpsc::Sender<TrackEvent>,
) {
    let ns = namespace.to_string();
    let name = pod_name.to_string();

    thread::spawn(move || {
        let key = format!("{}/{}", ns, name);

        loop {
            if !running.load(Ordering::SeqCst) || !alive.load(Ordering::SeqCst) {
                break;
            }

            let result = Command::new("kubectl")
                .args(["logs", "-f", "--tail=1000", &name, "-n", &ns])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match result {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        let mut in_traceback = false;

                        for line in reader.lines() {
                            if !running.load(Ordering::SeqCst) || !alive.load(Ordering::SeqCst) {
                                let _ = child.kill();
                                break;
                            }

                            match line {
                                Ok(text) => {
                                    if err_only && !should_show_line(&text, &mut in_traceback) {
                                        continue;
                                    }
                                    if tx
                                        .send(TrackEvent::LogLine {
                                            pod_key: key.clone(),
                                            text,
                                        })
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }
                    let _ = child.wait();
                }
                Err(_) => break,
            }

            if !running.load(Ordering::SeqCst) || !alive.load(Ordering::SeqCst) {
                break;
            }

            thread::sleep(Duration::from_secs(3));
        }

        active_pods.lock().unwrap().remove(&key);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pane(key: &str, n_lines: usize) -> PodPane {
        let alive = Arc::new(AtomicBool::new(true));
        let mut pane = PodPane::new(key.to_string(), Color::Cyan, alive);
        for i in 0..n_lines {
            pane.push_line(format!("line {i}"));
        }
        pane
    }

    #[test]
    fn test_pod_pane_new_defaults() {
        let alive = Arc::new(AtomicBool::new(true));
        let pane = PodPane::new("ns/pod".to_string(), Color::Green, alive);
        assert_eq!(pane.key, "ns/pod");
        assert!(pane.lines.is_empty());
        assert!(pane.is_following());
        assert_eq!(pane.scroll_offset(10), 0);
    }

    #[test]
    fn test_pod_pane_push_line() {
        let mut pane = make_pane("ns/pod", 0);
        pane.push_line("hello".to_string());
        pane.push_line("world".to_string());
        assert_eq!(pane.lines.len(), 2);
        assert_eq!(pane.lines[0], "hello");
        assert_eq!(pane.lines[1], "world");
    }

    #[test]
    fn test_pod_pane_push_line_caps_at_max() {
        let mut pane = make_pane("ns/pod", MAX_LOG_LINES);
        assert_eq!(pane.lines.len(), MAX_LOG_LINES);
        pane.push_line("overflow".to_string());
        assert_eq!(pane.lines.len(), MAX_LOG_LINES);
        assert_eq!(pane.lines.back().unwrap(), "overflow");
        assert_eq!(pane.lines.front().unwrap(), "line 1");
    }

    #[test]
    fn test_pod_pane_scroll_offset_following() {
        let pane = make_pane("ns/pod", 100);
        assert_eq!(pane.scroll_offset(20), 80);
    }

    #[test]
    fn test_pod_pane_scroll_offset_fewer_lines_than_height() {
        let pane = make_pane("ns/pod", 5);
        assert_eq!(pane.scroll_offset(20), 0);
    }

    #[test]
    fn test_pod_pane_scroll_offset_scrolled_up() {
        let mut pane = make_pane("ns/pod", 100);
        pane.scroll_up = Some(10);
        assert_eq!(pane.scroll_offset(20), 70);
    }

    #[test]
    fn test_pod_pane_scroll_offset_scrolled_to_top() {
        let mut pane = make_pane("ns/pod", 100);
        pane.scroll_up = Some(80);
        assert_eq!(pane.scroll_offset(20), 0);
    }

    #[test]
    fn test_pod_pane_scroll_offset_scroll_beyond_top_saturates() {
        let mut pane = make_pane("ns/pod", 100);
        pane.scroll_up = Some(200);
        assert_eq!(pane.scroll_offset(20), 0);
    }

    #[test]
    fn test_pod_pane_is_following() {
        let mut pane = make_pane("ns/pod", 10);
        assert!(pane.is_following());
        pane.scroll_up = Some(5);
        assert!(!pane.is_following());
        pane.scroll_up = None;
        assert!(pane.is_following());
    }

    #[test]
    fn test_rebuild_pane_index() {
        let panes = vec![
            make_pane("ns/a", 0),
            make_pane("ns/b", 0),
            make_pane("ns/c", 0),
        ];
        let index = rebuild_pane_index(&panes);
        assert_eq!(index.get("ns/a"), Some(&0));
        assert_eq!(index.get("ns/b"), Some(&1));
        assert_eq!(index.get("ns/c"), Some(&2));
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn test_rebuild_pane_index_after_removal() {
        let mut panes = vec![
            make_pane("ns/a", 0),
            make_pane("ns/b", 0),
            make_pane("ns/c", 0),
        ];
        panes.remove(1);
        let index = rebuild_pane_index(&panes);
        assert_eq!(index.get("ns/a"), Some(&0));
        assert_eq!(index.get("ns/c"), Some(&1));
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn test_handle_normal_mode_quit() {
        let running = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut selected = 0;
        let mut input_mode = false;
        let mut expanded = false;
        let mut panes = vec![make_pane("ns/a", 10)];
        let mut pane_index = rebuild_pane_index(&panes);

        let quit = handle_normal_mode(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );

        assert!(quit);
        assert!(!running.load(Ordering::SeqCst));
    }

    #[test]
    fn test_handle_normal_mode_tab_cycles() {
        let running = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut selected = 0;
        let mut input_mode = false;
        let mut expanded = false;
        let mut panes = vec![make_pane("ns/a", 0), make_pane("ns/b", 0), make_pane("ns/c", 0)];
        let mut pane_index = rebuild_pane_index(&panes);

        for expected in [1, 2, 0] {
            handle_normal_mode(
                KeyCode::Tab,
                KeyModifiers::NONE,
                &mut selected,
                &mut input_mode,
                &mut expanded,
                &mut panes,
                &mut pane_index,
                &running,
                &closed,
                20,
            );
            assert_eq!(selected, expected);
        }
    }

    #[test]
    fn test_handle_normal_mode_expand_toggle() {
        let running = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut selected = 0;
        let mut input_mode = false;
        let mut expanded = false;
        let mut panes = vec![make_pane("ns/a", 0)];
        let mut pane_index = rebuild_pane_index(&panes);

        handle_normal_mode(
            KeyCode::Char('f'),
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );
        assert!(expanded);

        handle_normal_mode(
            KeyCode::Char('f'),
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );
        assert!(!expanded);
    }

    #[test]
    fn test_handle_normal_mode_esc_collapses() {
        let running = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut selected = 0;
        let mut input_mode = false;
        let mut expanded = true;
        let mut panes = vec![make_pane("ns/a", 0)];
        let mut pane_index = rebuild_pane_index(&panes);

        handle_normal_mode(
            KeyCode::Esc,
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );
        assert!(!expanded);
    }

    #[test]
    fn test_handle_normal_mode_delete_pane() {
        let running = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut selected = 1;
        let mut input_mode = false;
        let mut expanded = false;
        let mut panes = vec![make_pane("ns/a", 0), make_pane("ns/b", 0), make_pane("ns/c", 0)];
        let mut pane_index = rebuild_pane_index(&panes);

        handle_normal_mode(
            KeyCode::Char('d'),
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );

        assert_eq!(panes.len(), 2);
        assert_eq!(panes[0].key, "ns/a");
        assert_eq!(panes[1].key, "ns/c");
        assert!(closed.lock().unwrap().contains("ns/b"));
        assert_eq!(selected, 1);
    }

    #[test]
    fn test_handle_normal_mode_delete_last_pane_clamps_selected() {
        let running = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut selected = 2;
        let mut input_mode = false;
        let mut expanded = false;
        let mut panes = vec![make_pane("ns/a", 0), make_pane("ns/b", 0), make_pane("ns/c", 0)];
        let mut pane_index = rebuild_pane_index(&panes);

        handle_normal_mode(
            KeyCode::Char('d'),
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );

        assert_eq!(panes.len(), 2);
        assert_eq!(selected, 1);
    }

    #[test]
    fn test_handle_normal_mode_scroll_up_down() {
        let running = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut selected = 0;
        let mut input_mode = false;
        let mut expanded = false;
        let mut panes = vec![make_pane("ns/a", 100)];
        let mut pane_index = rebuild_pane_index(&panes);

        assert!(panes[0].is_following());

        handle_normal_mode(
            KeyCode::Up,
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );
        assert_eq!(panes[0].scroll_up, Some(1));

        handle_normal_mode(
            KeyCode::Up,
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );
        assert_eq!(panes[0].scroll_up, Some(2));

        handle_normal_mode(
            KeyCode::Down,
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );
        assert_eq!(panes[0].scroll_up, Some(1));

        handle_normal_mode(
            KeyCode::Down,
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );
        assert!(panes[0].is_following());
    }

    #[test]
    fn test_handle_normal_mode_end_resumes_follow() {
        let running = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut selected = 0;
        let mut input_mode = false;
        let mut expanded = false;
        let mut panes = vec![make_pane("ns/a", 100)];
        let mut pane_index = rebuild_pane_index(&panes);
        panes[0].scroll_up = Some(50);

        handle_normal_mode(
            KeyCode::End,
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );
        assert!(panes[0].is_following());
    }

    #[test]
    fn test_handle_normal_mode_home_scrolls_to_top() {
        let running = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut selected = 0;
        let mut input_mode = false;
        let mut expanded = false;
        let mut panes = vec![make_pane("ns/a", 100)];
        let mut pane_index = rebuild_pane_index(&panes);

        handle_normal_mode(
            KeyCode::Home,
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );
        assert_eq!(panes[0].scroll_up, Some(99));
        assert_eq!(panes[0].scroll_offset(20), 0);
    }

    #[test]
    fn test_handle_normal_mode_input_mode() {
        let running = Arc::new(AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut selected = 0;
        let mut input_mode = false;
        let mut expanded = false;
        let mut panes = vec![make_pane("ns/a", 0)];
        let mut pane_index = rebuild_pane_index(&panes);

        handle_normal_mode(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
            &mut selected,
            &mut input_mode,
            &mut expanded,
            &mut panes,
            &mut pane_index,
            &running,
            &closed,
            20,
        );
        assert!(input_mode);
    }

    #[test]
    fn test_scroll_to_scrollbar_pos_outside_scrollbar_col() {
        let rect = Rect::new(0, 0, 80, 20);
        let mut pane = make_pane("ns/a", 100);
        scroll_to_scrollbar_pos(10, 5, &rect, Some(&mut pane));
        assert!(pane.is_following());
    }

    #[test]
    fn test_scroll_to_scrollbar_pos_on_scrollbar() {
        let rect = Rect::new(0, 0, 80, 22);
        let mut pane = make_pane("ns/a", 100);
        scroll_to_scrollbar_pos(79, 1, &rect, Some(&mut pane));
        assert!(!pane.is_following());
    }

    #[test]
    fn test_scroll_to_scrollbar_pos_bottom_of_scrollbar() {
        let rect = Rect::new(0, 0, 80, 22);
        let mut pane = make_pane("ns/a", 100);
        let inner_height = 20usize;
        let max_scroll = 100 - inner_height;
        scroll_to_scrollbar_pos(79, 20, &rect, Some(&mut pane));
        let expected_target = (19 * max_scroll) / inner_height;
        let expected_up = max_scroll.saturating_sub(expected_target);
        assert_eq!(pane.scroll_up, if expected_up == 0 { None } else { Some(expected_up) });
    }

    #[test]
    fn test_handle_mouse_scroll_up() {
        let mut selected = 0;
        let mut panes = vec![make_pane("ns/a", 100), make_pane("ns/b", 50)];
        let pane_rects = vec![
            (0, Rect::new(0, 0, 80, 20)),
            (1, Rect::new(0, 20, 80, 20)),
        ];

        let mouse = crossterm::event::MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 10,
            row: 25,
            modifiers: KeyModifiers::NONE,
        };
        handle_mouse_event(mouse, &mut selected, &mut panes, &pane_rects);

        assert_eq!(selected, 1);
        assert_eq!(panes[1].scroll_up, Some(3));
    }

    #[test]
    fn test_handle_mouse_click_selects_pane() {
        let mut selected = 0;
        let mut panes = vec![make_pane("ns/a", 10), make_pane("ns/b", 10)];
        let pane_rects = vec![
            (0, Rect::new(0, 0, 80, 20)),
            (1, Rect::new(0, 20, 80, 20)),
        ];

        let mouse = crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 25,
            modifiers: KeyModifiers::NONE,
        };
        handle_mouse_event(mouse, &mut selected, &mut panes, &pane_rects);

        assert_eq!(selected, 1);
    }
}
