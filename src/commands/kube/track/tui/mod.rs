mod event;
mod kube;
mod shared;
mod state;
mod theme;
mod traits;

use ansi_to_tui::IntoText;
use anyhow::Result;
use crossterm::event::{Event, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use super::PodInfo;
use event::TrackEvent;
use regex::Regex;
use shared::SharedState;
use state::{KeyAction, PodPane, TuiState};
use theme::Theme;
use traits::{LogStreamSpawnOpts, LogStreamSpawner, PodDiscovery, PatternToRegex};

const MAIN_LAYOUT: [Constraint; 2] =
    [Constraint::Min(0), Constraint::Length(1)];

fn spawn_opts(shared: &SharedState, alive: Arc<AtomicBool>) -> LogStreamSpawnOpts {
    LogStreamSpawnOpts {
        err_only: shared.err_only,
        running: shared.running.clone(),
        alive,
        active_pods: shared.active_pods.clone(),
        tx: shared.tx.clone(),
    }
}

fn try_spawn_pods(
    shared: &SharedState,
    pods: Vec<PodInfo>,
    spawner: &Arc<dyn LogStreamSpawner>,
) {
    for pod in pods {
        let key = pod.key();
        if shared.closed_pods.lock().unwrap().contains(&key) {
            continue;
        }
        let should_spawn = {
            let mut active = shared.active_pods.lock().unwrap();
            if active.contains(&key) {
                false
            } else {
                active.insert(key.clone());
                true
            }
        };
        if should_spawn {
            let alive = Arc::new(AtomicBool::new(true));
            spawner.spawn(&pod, spawn_opts(shared, alive.clone()));
            let _ = shared.tx.send(TrackEvent::NewPod { pod, alive });
        }
    }
}

fn key_hint(key: &str, desc: &str, color: Color) -> Vec<Span<'static>> {
    vec![
        Span::styled(key.to_string(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled(format!(": {}  ", desc), Style::default().fg(Color::White)),
    ]
}

fn ratio_chunks(area: Rect, count: usize) -> Vec<Rect> {
    let n = count.max(1) as u32;
    let constraints: Vec<Constraint> = (0..count).map(|_| Constraint::Ratio(1, n)).collect();
    Layout::vertical(constraints).split(area).to_vec()
}

fn start_poll_loop(
    shared: Arc<SharedState>,
    discovery: Arc<dyn PodDiscovery>,
    spawner: Arc<dyn LogStreamSpawner>,
) {
    thread::spawn(move || {
        while shared.running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(5));
            if !shared.running.load(Ordering::SeqCst) {
                break;
            }
            let current_regexes = shared.clone_regexes();
            if let Ok(new_pods) = discovery.find_matching_pods(&current_regexes) {
                try_spawn_pods(&shared, new_pods, &spawner);
            }
        }
    });
}

pub struct TrackTui {
    state: TuiState,
    shared: Arc<SharedState>,
    color_counter: Arc<AtomicUsize>,
    rx: mpsc::Receiver<TrackEvent>,
    theme: Theme,
    discovery: Arc<dyn PodDiscovery>,
    spawner: Arc<dyn LogStreamSpawner>,
    pattern_to_regex: Arc<dyn PatternToRegex>,
    layout_available_height: u16,
    layout_page_size: usize,
}

impl TrackTui {
    pub fn with_deps(
        pods: Vec<PodInfo>,
        shared: SharedState,
        rx: mpsc::Receiver<TrackEvent>,
        discovery: Arc<dyn PodDiscovery>,
        spawner: Arc<dyn LogStreamSpawner>,
        pattern_to_regex: Arc<dyn PatternToRegex>,
        theme: Theme,
    ) -> Self {
        let shared = Arc::new(shared);
        start_poll_loop(shared.clone(), discovery.clone(), spawner.clone());

        let color_counter = Arc::new(AtomicUsize::new(0));
        let mut state = TuiState::new();

        for pod in &pods {
            let key = pod.key();
            let cidx = color_counter.fetch_add(1, Ordering::SeqCst);
            let color = theme.pane_color(cidx);
            let alive = Arc::new(AtomicBool::new(true));
            state.add_pane(PodPane::new(key.clone(), color, alive.clone()));
            shared.active_pods.lock().unwrap().insert(key.clone());
            spawner.spawn(pod, spawn_opts(&shared, alive));
        }

        Self {
            state,
            shared,
            color_counter,
            rx,
            theme,
            discovery,
            spawner,
            pattern_to_regex,
            layout_available_height: 0,
            layout_page_size: 0,
        }
    }

    fn add_pattern(&self, pattern: &str) {
        let new_regex = self.pattern_to_regex.build(pattern);
        self.shared.add_regex(new_regex.clone());

        let shared = self.shared.clone();
        let discovery = self.discovery.clone();
        let spawner = self.spawner.clone();

        thread::spawn(move || {
            if let Ok(pods) = discovery.find_matching_pods(&[new_regex]) {
                try_spawn_pods(&shared, pods, &spawner);
            }
        });
    }

    fn process_track_events(&mut self) {
        while let Ok(evt) = self.rx.try_recv() {
            match evt {
                TrackEvent::LogLine { pod_key, text } => {
                    if let Some(&idx) = self.state.pane_index.get(&pod_key) {
                        self.state.panes[idx].push_line(text);
                    }
                }
                TrackEvent::NewPod { pod, alive } => {
                    let key = pod.key();
                    if !self.state.pane_index.contains_key(&key) {
                        let cidx = self.color_counter.fetch_add(1, Ordering::SeqCst);
                        let color = self.theme.pane_color(cidx);
                        self.state.add_pane(PodPane::new(key, color, alive));
                    }
                }
            }
        }
    }

    fn update_pane_rects(&mut self, term_size: Rect, available_height: u16) {
        let main_layout = Layout::vertical(MAIN_LAYOUT).split(term_size);
        let visible_indices = self.state.visible_indices(available_height);
        let chunks = ratio_chunks(main_layout[0], visible_indices.len());
        self.state.pane_rects = visible_indices
            .iter()
            .zip(chunks.iter())
            .map(|(&i, r)| (i, *r))
            .collect();
    }

    fn render_frame(&self, frame: &mut Frame, total_tabs: usize, available_height: u16) {
        let main_chunks = Layout::vertical(MAIN_LAYOUT).split(frame.area());

        if !self.state.panes.is_empty() {
            let visible_indices = self.state.visible_indices(available_height);
            let visible: Vec<(usize, &PodPane)> = visible_indices
                .iter()
                .filter_map(|&i| self.state.panes.get(i).map(|p| (i, p)))
                .collect();
            let chunks = ratio_chunks(main_chunks[0], visible.len());

            for (ci, (i, pane)) in visible.iter().enumerate() {
                let is_selected = *i == self.state.selected;
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
                    format!(" {} [SCROLLED] ", pane.key)
                };

                let title_color = self.theme.title_color(*i);
                let title_style = if !pane.is_following() {
                    Style::default()
                        .fg(Color::Rgb(0xff, 0xcc, 0x00))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(title_color)
                        .add_modifier(Modifier::BOLD)
                };

                let block = Block::bordered()
                    .title(title)
                    .title_style(title_style)
                    .border_style(border_style);

                let inner_height = chunks[ci].height.saturating_sub(2) as usize;
                let scroll_offset = pane.scroll_offset(inner_height) as usize;

                let visible_end = (scroll_offset + inner_height).min(pane.lines.len());
                let visible_slice: String = pane
                    .lines
                    .iter()
                    .skip(scroll_offset)
                    .take(visible_end - scroll_offset)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n");
                let text = visible_slice
                    .as_bytes()
                    .into_text()
                    .unwrap_or_else(|_| Text::raw(&visible_slice));
                let paragraph = Paragraph::new(text)
                    .block(block)
                    .wrap(Wrap { trim: true });

                frame.render_widget(paragraph, chunks[ci]);

                if pane.lines.len() > inner_height {
                    let max_scroll = pane.lines.len().saturating_sub(inner_height);
                    let mut scrollbar_state =
                        ScrollbarState::new(max_scroll).position(scroll_offset);
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

        let status_line = if self.state.input_mode {
            let mut spans = vec![
                Span::styled(
                    " Pattern: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(self.state.input_buffer.as_str()),
                Span::styled("\u{2588}", Style::default().fg(Color::White)),
                Span::raw("  "),
            ];
            spans.extend(key_hint("Enter", "add", Color::Green));
            spans.extend(key_hint("Esc", "cancel", Color::Red));
            Line::from(spans)
        } else {
            let mut spans = vec![Span::raw(" ")];

            if total_tabs > 1 {
                spans.push(Span::styled(
                    format!("[{}/{}]", self.state.current_tab + 1, total_tabs),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw("  "));
                spans.extend(key_hint("\u{2190}\u{2192}", "tabs", Color::LightCyan));
            }
            spans.extend(key_hint("Tab", "switch", Color::LightCyan));
            spans.extend(key_hint("\u{2191}\u{2193}", "scroll", Color::LightCyan));
            spans.extend(key_hint("End", "follow", Color::LightCyan));
            spans.extend(key_hint(
                "f",
                if self.state.expanded { "collapse" } else { "expand" },
                Color::LightCyan,
            ));
            spans.extend(key_hint("a", "add pod", Color::LightGreen));
            spans.extend(key_hint("d", "close pane", Color::LightRed));
            spans.extend(key_hint("D", "close tab", Color::LightRed));
            spans.extend(key_hint(
                "m",
                if self.state.mouse_captured {
                    "select text"
                } else {
                    "mouse mode"
                },
                Color::LightCyan,
            ));
            if !self.state.mouse_captured {
                spans.extend(key_hint("Ctrl+\u{2191}\u{2193}", "scroll", Color::LightCyan));
            }
            spans.extend(key_hint("q", "quit", Color::LightYellow));

            Line::from(spans)
        };

        frame.render_widget(
            Paragraph::new(status_line).style(Style::default().bg(Color::Rgb(30, 30, 30))),
            main_chunks[1],
        );
    }
}

impl crate::tui::TuiApp for TrackTui {
    fn update_layout(&mut self, term_size: Rect, available_height: u16) {
        self.layout_available_height = available_height;
        self.update_pane_rects(term_size, available_height);
        let per_tab = self.state.max_panes_per_tab(available_height);
        let tab_start = self.state.current_tab * per_tab;
        let tab_end = (tab_start + per_tab).min(self.state.panes.len());
        let visible_count = tab_end.saturating_sub(tab_start).max(1);
        let pane_h = available_height / visible_count as u16;
        self.layout_page_size = pane_h.saturating_sub(2) as usize;
    }

    fn render(&self, frame: &mut Frame) {
        let total_tabs = self.state.total_tabs(self.layout_available_height);
        self.render_frame(frame, total_tabs, self.layout_available_height);
    }

    fn poll_interval(&self) -> Duration {
        Duration::from_millis(50)
    }

    fn process_background(&mut self) {
        self.process_track_events();
    }

    fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if self.state.input_mode {
                    let action = self.state.handle_input_key(key_event.code);
                    if let KeyAction::AddPattern(pattern) = action {
                        self.add_pattern(&pattern);
                    }
                } else {
                    let action = self.state.handle_key(
                        key_event.code,
                        key_event.modifiers,
                        self.layout_page_size,
                        self.layout_available_height,
                        &self.shared.running,
                        &self.shared.closed_pods,
                    );
                    if let KeyAction::Quit = action {
                        return Ok(true);
                    }
                }
            }
            Event::Mouse(mouse_event) if !self.state.input_mode => {
                self.state.handle_mouse(mouse_event);
            }
            _ => {}
        }
        Ok(false)
    }
}

pub fn run(pods: Vec<PodInfo>, regexes: Vec<Regex>, err_only: bool) -> Result<()> {
    run_with(
        pods,
        regexes,
        err_only,
        Arc::new(kube::KubePodDiscovery),
        Arc::new(kube::KubectlLogSpawner),
        Arc::new(kube::KubePatternToRegex),
        Theme::default(),
    )
}

pub fn run_with(
    pods: Vec<PodInfo>,
    initial_regexes: Vec<Regex>,
    err_only: bool,
    discovery: Arc<dyn PodDiscovery>,
    spawner: Arc<dyn LogStreamSpawner>,
    pattern_to_regex: Arc<dyn PatternToRegex>,
    theme: Theme,
) -> Result<()> {
    let (shared, rx) = SharedState::new(err_only, initial_regexes);
    let app = TrackTui::with_deps(
        pods,
        shared,
        rx,
        discovery,
        spawner,
        pattern_to_regex,
        theme,
    );
    crate::tui::run(app)
}
