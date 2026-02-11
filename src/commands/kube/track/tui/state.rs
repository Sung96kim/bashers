use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers, MouseButton, MouseEventKind,
};
use crossterm::ExecutableCommand;
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

pub const MAX_LOG_LINES: usize = 5000;
pub const MIN_PANE_HEIGHT: u16 = 12;

#[derive(Clone)]
pub struct PodPane {
    pub key: String,
    pub color: Color,
    pub lines: std::collections::VecDeque<String>,
    pub alive: Arc<std::sync::atomic::AtomicBool>,
    pub scroll_up: Option<usize>,
}

impl PodPane {
    pub fn new(
        key: String,
        color: Color,
        alive: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            key,
            color,
            lines: std::collections::VecDeque::with_capacity(MAX_LOG_LINES),
            alive,
            scroll_up: None,
        }
    }

    pub fn push_line(&mut self, line: String) {
        let was_at_max = self.lines.len() >= MAX_LOG_LINES;
        if was_at_max {
            self.lines.pop_front();
            if let Some(ref mut pos) = self.scroll_up {
                *pos = pos.saturating_sub(1);
            }
        }
        self.lines.push_back(line);
    }

    pub fn scroll_offset(&self, inner_height: usize) -> u16 {
        let auto = self.lines.len().saturating_sub(inner_height);
        match self.scroll_up {
            None => auto as u16,
            Some(pos) => (pos as u16).min(auto as u16),
        }
    }

    pub fn is_following(&self) -> bool {
        self.scroll_up.is_none()
    }

    pub fn scroll_up_by(&mut self, page_size: usize, delta: usize) {
        let auto = self.lines.len().saturating_sub(page_size);
        let current = self.scroll_up.unwrap_or(auto);
        self.scroll_up = Some(current.saturating_sub(delta));
    }

    pub fn scroll_down_by(&mut self, page_size: usize, delta: usize) {
        if let Some(pos) = self.scroll_up {
            let auto = self.lines.len().saturating_sub(page_size);
            if pos + delta >= auto {
                self.scroll_up = None;
            } else {
                self.scroll_up = Some(pos + delta);
            }
        }
    }

    pub fn scroll_home(&mut self) {
        if !self.lines.is_empty() {
            self.scroll_up = Some(0);
        }
    }

    pub fn scroll_end(&mut self) {
        self.scroll_up = None;
    }
}

pub enum KeyAction {
    Quit,
    AddPattern(String),
    None,
}

pub struct TuiState {
    pub selected: usize,
    pub current_tab: usize,
    pub expanded: bool,
    pub input_mode: bool,
    pub input_buffer: String,
    pub panes: Vec<PodPane>,
    pub pane_index: HashMap<String, usize>,
    pub pane_rects: Vec<(usize, Rect)>,
    last_click: Option<(usize, std::time::Instant)>,
    pub mouse_captured: bool,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            current_tab: 0,
            expanded: false,
            input_mode: false,
            input_buffer: String::new(),
            panes: Vec::new(),
            pane_index: HashMap::new(),
            pane_rects: vec![],
            last_click: None,
            mouse_captured: true,
        }
    }

    pub fn add_pane(&mut self, pane: PodPane) {
        self.pane_index.insert(pane.key.clone(), self.panes.len());
        self.panes.push(pane);
    }

    pub fn rebuild_index(&mut self) {
        self.pane_index = self
            .panes
            .iter()
            .enumerate()
            .map(|(i, p)| (p.key.clone(), i))
            .collect();
    }

    pub fn max_panes_per_tab(&self, available_height: u16) -> usize {
        (available_height / MIN_PANE_HEIGHT).max(1) as usize
    }

    pub fn total_tabs(&self, available_height: u16) -> usize {
        if self.panes.is_empty() {
            return 1;
        }
        let per_tab = self.max_panes_per_tab(available_height);
        self.panes.len().div_ceil(per_tab)
    }

    pub fn visible_indices(&self, available_height: u16) -> Vec<usize> {
        if self.expanded {
            return vec![self.selected];
        }
        let per_tab = self.max_panes_per_tab(available_height);
        let start = self.current_tab * per_tab;
        let end = (start + per_tab).min(self.panes.len());
        (start..end).collect()
    }

    pub fn ensure_selected_visible(&mut self, available_height: u16) {
        if self.panes.is_empty() {
            self.current_tab = 0;
            return;
        }
        let per_tab = self.max_panes_per_tab(available_height);
        let total = self.total_tabs(available_height);
        self.current_tab = self.current_tab.min(total.saturating_sub(1));
        let tab_for_selected = self.selected / per_tab;
        self.current_tab = tab_for_selected.min(total.saturating_sub(1));
    }

    pub(crate) fn scroll_to_scrollbar_pos(
        col: u16,
        row: u16,
        rect: &Rect,
        pane: Option<&mut PodPane>,
    ) {
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
                let target_pos = (click_pos * max_scroll) / inner_height.max(1);
                if target_pos >= max_scroll {
                    pane.scroll_up = None;
                } else {
                    pane.scroll_up = Some(target_pos);
                }
            }
        }
    }

    pub fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        let col = mouse.column;
        let row = mouse.row;

        let hit = self
            .pane_rects
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

        let scrollbar_col = rect.x + rect.width - 1;
        let on_scrollbar = col >= scrollbar_col.saturating_sub(1);
        let inner_h = rect.height.saturating_sub(2) as usize;

        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.selected = pane_idx;
                if let Some(pane) = self.panes.get_mut(pane_idx) {
                    pane.scroll_up_by(inner_h, 3);
                }
            }
            MouseEventKind::ScrollDown => {
                self.selected = pane_idx;
                if let Some(pane) = self.panes.get_mut(pane_idx) {
                    pane.scroll_down_by(inner_h, 3);
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                let now = std::time::Instant::now();
                let is_double = self
                    .last_click
                    .map(|(prev_idx, prev_time)| {
                        prev_idx == pane_idx && now.duration_since(prev_time).as_millis() < 400
                    })
                    .unwrap_or(false);

                self.selected = pane_idx;

                if is_double {
                    self.expanded = !self.expanded;
                    self.last_click = None;
                } else if on_scrollbar {
                    Self::scroll_to_scrollbar_pos(
                        col,
                        row,
                        &rect,
                        self.panes.get_mut(pane_idx),
                    );
                    self.last_click = Some((pane_idx, now));
                } else {
                    self.last_click = Some((pane_idx, now));
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if on_scrollbar {
                    Self::scroll_to_scrollbar_pos(
                        col,
                        row,
                        &rect,
                        self.panes.get_mut(pane_idx),
                    );
                }
            }
            _ => {}
        }
    }

    pub fn handle_input_key(&mut self, code: KeyCode) -> KeyAction {
        match code {
            KeyCode::Enter => {
                if !self.input_buffer.is_empty() {
                    let pattern = self.input_buffer.clone();
                    self.input_buffer.clear();
                    self.input_mode = false;
                    return KeyAction::AddPattern(pattern);
                }
            }
            KeyCode::Esc => {
                self.input_buffer.clear();
                self.input_mode = false;
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        KeyAction::None
    }

    pub fn handle_key(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
        page_size: usize,
        available_height: u16,
        running: &Arc<std::sync::atomic::AtomicBool>,
        closed_pods: &Arc<Mutex<HashSet<String>>>,
    ) -> KeyAction {
        let ctrl_scroll = modifiers.contains(KeyModifiers::CONTROL);
        match code {
            KeyCode::Char('q') => {
                running.store(false, Ordering::SeqCst);
                return KeyAction::Quit;
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                running.store(false, Ordering::SeqCst);
                return KeyAction::Quit;
            }
            KeyCode::Char('f') => {
                self.expanded = !self.expanded;
            }
            KeyCode::Esc => {
                if self.expanded {
                    self.expanded = false;
                }
            }
            KeyCode::Left => {
                if self.current_tab > 0 {
                    let per_tab = self.max_panes_per_tab(available_height);
                    self.current_tab -= 1;
                    self.selected = self.current_tab * per_tab;
                }
            }
            KeyCode::Right => {
                let total = self.total_tabs(available_height);
                if self.current_tab + 1 < total {
                    let per_tab = self.max_panes_per_tab(available_height);
                    self.current_tab += 1;
                    self.selected = (self.current_tab * per_tab).min(self.panes.len().saturating_sub(1));
                }
            }
            KeyCode::Tab | KeyCode::Char('j') => {
                if !self.panes.is_empty() {
                    self.selected = (self.selected + 1) % self.panes.len();
                    self.ensure_selected_visible(available_height);
                }
            }
            KeyCode::BackTab | KeyCode::Char('k') => {
                if !self.panes.is_empty() {
                    self.selected = self
                        .selected
                        .checked_sub(1)
                        .unwrap_or(self.panes.len() - 1);
                    self.ensure_selected_visible(available_height);
                }
            }
            KeyCode::Up if ctrl_scroll => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_up_by(page_size, 1);
                }
            }
            KeyCode::Down if ctrl_scroll => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_down_by(page_size, 1);
                }
            }
            KeyCode::PageUp if ctrl_scroll => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_up_by(page_size, page_size);
                }
            }
            KeyCode::PageDown if ctrl_scroll => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_down_by(page_size, page_size);
                }
            }
            KeyCode::Home if ctrl_scroll => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_home();
                }
            }
            KeyCode::End if ctrl_scroll => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_end();
                }
            }
            KeyCode::Up => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_up_by(page_size, 1);
                }
            }
            KeyCode::Down => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_down_by(page_size, 1);
                }
            }
            KeyCode::PageUp => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_up_by(page_size, page_size);
                }
            }
            KeyCode::PageDown => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_down_by(page_size, page_size);
                }
            }
            KeyCode::Home => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_home();
                }
            }
            KeyCode::End => {
                if let Some(pane) = self.panes.get_mut(self.selected) {
                    pane.scroll_end();
                }
            }
            KeyCode::Char('m') => {
                self.mouse_captured = !self.mouse_captured;
                if self.mouse_captured {
                    let _ = std::io::stdout().execute(EnableMouseCapture);
                } else {
                    let _ = std::io::stdout().execute(DisableMouseCapture);
                }
            }
            KeyCode::Char('a') => {
                self.input_mode = true;
            }
            KeyCode::Char('d') => {
                if !self.panes.is_empty() {
                    let removed = self.panes.remove(self.selected);
                    removed.alive.store(false, Ordering::SeqCst);
                    closed_pods.lock().unwrap().insert(removed.key.clone());
                    self.rebuild_index();
                    if self.panes.is_empty() {
                        self.selected = 0;
                    } else {
                        self.selected = self.selected.min(self.panes.len() - 1);
                    }
                    self.ensure_selected_visible(available_height);
                }
            }
            KeyCode::Char('D') => {
                let per_tab = self.max_panes_per_tab(available_height);
                let start = self.current_tab * per_tab;
                let end = (start + per_tab).min(self.panes.len());
                if start < end {
                    let indices: Vec<usize> = (start..end).rev().collect();
                    for idx in indices {
                        let pane = self.panes.remove(idx);
                        pane.alive.store(false, Ordering::SeqCst);
                        closed_pods.lock().unwrap().insert(pane.key.clone());
                    }
                    self.rebuild_index();
                    if self.panes.is_empty() {
                        self.selected = 0;
                        self.current_tab = 0;
                    } else {
                        self.current_tab = self
                            .current_tab
                            .min(self.total_tabs(available_height).saturating_sub(1));
                        self.selected = (self.current_tab * per_tab)
                            .min(self.panes.len().saturating_sub(1));
                    }
                }
            }
            _ => {}
        }
        KeyAction::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn make_pane(key: &str, n_lines: usize) -> PodPane {
        let alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let mut pane = PodPane::new(key.to_string(), Color::Cyan, alive);
        for i in 0..n_lines {
            pane.push_line(format!("line {i}"));
        }
        pane
    }

    fn make_state(keys: &[&str], lines_per_pane: usize) -> TuiState {
        let mut state = TuiState::new();
        for key in keys {
            state.add_pane(make_pane(key, lines_per_pane));
        }
        state
    }

    fn press_key(
        state: &mut TuiState,
        code: KeyCode,
        running: &Arc<std::sync::atomic::AtomicBool>,
        closed: &Arc<Mutex<HashSet<String>>>,
    ) -> KeyAction {
        state.handle_key(code, KeyModifiers::NONE, 20, 48, running, closed)
    }

    #[test]
    fn test_pod_pane_new_defaults() {
        let alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
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
    fn test_pod_pane_scroll_offset_scrolled() {
        let mut pane = make_pane("ns/pod", 100);
        pane.scroll_up = Some(10);
        assert_eq!(pane.scroll_offset(20), 10);
    }

    #[test]
    fn test_pod_pane_scroll_offset_at_top() {
        let mut pane = make_pane("ns/pod", 100);
        pane.scroll_up = Some(0);
        assert_eq!(pane.scroll_offset(20), 0);
    }

    #[test]
    fn test_pod_pane_scroll_offset_clamped_to_auto() {
        let mut pane = make_pane("ns/pod", 100);
        pane.scroll_up = Some(200);
        assert_eq!(pane.scroll_offset(20), 80);
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
    fn test_tui_state_add_pane_and_rebuild() {
        let mut state = make_state(&["ns/a", "ns/b", "ns/c"], 0);
        assert_eq!(state.pane_index.get("ns/a"), Some(&0));
        assert_eq!(state.pane_index.get("ns/b"), Some(&1));
        assert_eq!(state.pane_index.get("ns/c"), Some(&2));
        assert_eq!(state.pane_index.len(), 3);

        state.panes.remove(1);
        state.rebuild_index();
        assert_eq!(state.pane_index.get("ns/a"), Some(&0));
        assert_eq!(state.pane_index.get("ns/c"), Some(&1));
        assert_eq!(state.pane_index.len(), 2);
    }

    #[test]
    fn test_handle_key_quit() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["ns/a"], 10);

        let action = press_key(&mut state, KeyCode::Char('q'), &running, &closed);
        assert!(matches!(action, KeyAction::Quit));
        assert!(!running.load(Ordering::SeqCst));
    }

    #[test]
    fn test_handle_key_tab_cycles() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["ns/a", "ns/b", "ns/c"], 0);

        for expected in [1, 2, 0] {
            press_key(&mut state, KeyCode::Tab, &running, &closed);
            assert_eq!(state.selected, expected);
        }
    }

    #[test]
    fn test_handle_key_expand_toggle() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["ns/a"], 0);

        press_key(&mut state, KeyCode::Char('f'), &running, &closed);
        assert!(state.expanded);
        press_key(&mut state, KeyCode::Char('f'), &running, &closed);
        assert!(!state.expanded);
    }

    #[test]
    fn test_handle_key_esc_collapses() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["ns/a"], 0);
        state.expanded = true;

        press_key(&mut state, KeyCode::Esc, &running, &closed);
        assert!(!state.expanded);
    }

    #[test]
    fn test_handle_key_delete_pane() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["ns/a", "ns/b", "ns/c"], 0);
        state.selected = 1;

        press_key(&mut state, KeyCode::Char('d'), &running, &closed);

        assert_eq!(state.panes.len(), 2);
        assert_eq!(state.panes[0].key, "ns/a");
        assert_eq!(state.panes[1].key, "ns/c");
        assert!(closed.lock().unwrap().contains("ns/b"));
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn test_handle_key_delete_last_pane_clamps_selected() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["ns/a", "ns/b", "ns/c"], 0);
        state.selected = 2;

        press_key(&mut state, KeyCode::Char('d'), &running, &closed);

        assert_eq!(state.panes.len(), 2);
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn test_handle_key_scroll_up_down() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["ns/a"], 100);

        assert!(state.panes[0].is_following());

        press_key(&mut state, KeyCode::Up, &running, &closed);
        assert_eq!(state.panes[0].scroll_up, Some(79));

        press_key(&mut state, KeyCode::Up, &running, &closed);
        assert_eq!(state.panes[0].scroll_up, Some(78));

        press_key(&mut state, KeyCode::Down, &running, &closed);
        assert_eq!(state.panes[0].scroll_up, Some(79));

        press_key(&mut state, KeyCode::Down, &running, &closed);
        assert!(state.panes[0].is_following());
    }

    #[test]
    fn test_handle_key_end_resumes_follow() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["ns/a"], 100);
        state.panes[0].scroll_up = Some(50);

        press_key(&mut state, KeyCode::End, &running, &closed);
        assert!(state.panes[0].is_following());
    }

    #[test]
    fn test_handle_key_home_scrolls_to_top() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["ns/a"], 100);

        press_key(&mut state, KeyCode::Home, &running, &closed);
        assert_eq!(state.panes[0].scroll_up, Some(0));
        assert_eq!(state.panes[0].scroll_offset(20), 0);
    }

    #[test]
    fn test_handle_key_input_mode() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["ns/a"], 0);

        press_key(&mut state, KeyCode::Char('a'), &running, &closed);
        assert!(state.input_mode);
    }

    #[test]
    fn test_scroll_to_scrollbar_pos_outside_scrollbar_col() {
        let rect = Rect::new(0, 0, 80, 20);
        let mut pane = make_pane("ns/a", 100);
        TuiState::scroll_to_scrollbar_pos(10, 5, &rect, Some(&mut pane));
        assert!(pane.is_following());
    }

    #[test]
    fn test_scroll_to_scrollbar_pos_on_scrollbar() {
        let rect = Rect::new(0, 0, 80, 22);
        let mut pane = make_pane("ns/a", 100);
        TuiState::scroll_to_scrollbar_pos(79, 1, &rect, Some(&mut pane));
        assert!(!pane.is_following());
    }

    #[test]
    fn test_scroll_to_scrollbar_pos_bottom_of_scrollbar() {
        let rect = Rect::new(0, 0, 80, 22);
        let mut pane = make_pane("ns/a", 100);
        let inner_height = 20usize;
        let max_scroll = 100 - inner_height;
        TuiState::scroll_to_scrollbar_pos(79, 20, &rect, Some(&mut pane));
        let expected_pos = (19 * max_scroll) / inner_height;
        assert_eq!(pane.scroll_up, Some(expected_pos));
    }

    #[test]
    fn test_handle_mouse_scroll_up() {
        let mut state = make_state(&["ns/a", "ns/b"], 0);
        for _ in 0..100 {
            state.panes[0].push_line("x".to_string());
        }
        for _ in 0..50 {
            state.panes[1].push_line("x".to_string());
        }
        state.pane_rects = vec![
            (0, Rect::new(0, 0, 80, 20)),
            (1, Rect::new(0, 20, 80, 20)),
        ];

        let mouse = crossterm::event::MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 10,
            row: 25,
            modifiers: KeyModifiers::NONE,
        };
        state.handle_mouse(mouse);

        assert_eq!(state.selected, 1);
        let inner_h = 18;
        let auto = 50usize.saturating_sub(inner_h);
        assert_eq!(state.panes[1].scroll_up, Some(auto.saturating_sub(3)));
    }

    #[test]
    fn test_handle_mouse_click_selects_pane() {
        let mut state = make_state(&["ns/a", "ns/b"], 10);
        state.pane_rects = vec![
            (0, Rect::new(0, 0, 80, 20)),
            (1, Rect::new(0, 20, 80, 20)),
        ];

        let mouse = crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 25,
            modifiers: KeyModifiers::NONE,
        };
        state.handle_mouse(mouse);

        assert_eq!(state.selected, 1);
    }

    #[test]
    fn test_max_panes_per_tab() {
        let state = TuiState::new();
        assert_eq!(state.max_panes_per_tab(48), 4);
        assert_eq!(state.max_panes_per_tab(24), 2);
        assert_eq!(state.max_panes_per_tab(12), 1);
        assert_eq!(state.max_panes_per_tab(6), 1);
    }

    #[test]
    fn test_total_tabs() {
        let state = make_state(&["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"], 0);
        assert_eq!(state.total_tabs(48), 3);
        assert_eq!(state.total_tabs(24), 5);
        assert_eq!(state.total_tabs(120), 1);
    }

    #[test]
    fn test_total_tabs_empty() {
        let state = TuiState::new();
        assert_eq!(state.total_tabs(48), 1);
    }

    #[test]
    fn test_visible_indices_first_tab() {
        let state = make_state(&["a", "b", "c", "d", "e", "f", "g", "h"], 0);
        let vis = state.visible_indices(48);
        assert_eq!(vis, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_visible_indices_second_tab() {
        let mut state = make_state(&["a", "b", "c", "d", "e", "f", "g", "h"], 0);
        state.current_tab = 1;
        let vis = state.visible_indices(48);
        assert_eq!(vis, vec![4, 5, 6, 7]);
    }

    #[test]
    fn test_visible_indices_last_tab_partial() {
        let mut state = make_state(&["a", "b", "c", "d", "e"], 0);
        state.current_tab = 1;
        let vis = state.visible_indices(48);
        assert_eq!(vis, vec![4]);
    }

    #[test]
    fn test_visible_indices_expanded_overrides_tabs() {
        let mut state = make_state(&["a", "b", "c", "d", "e"], 0);
        state.selected = 3;
        state.expanded = true;
        let vis = state.visible_indices(48);
        assert_eq!(vis, vec![3]);
    }

    #[test]
    fn test_ensure_selected_visible() {
        let mut state = make_state(&["a", "b", "c", "d", "e", "f", "g", "h"], 0);
        state.selected = 5;
        state.ensure_selected_visible(48);
        assert_eq!(state.current_tab, 1);
    }

    #[test]
    fn test_tab_navigation_left_right() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["a", "b", "c", "d", "e", "f", "g", "h"], 0);

        state.handle_key(KeyCode::Right, KeyModifiers::NONE, 20, 48, &running, &closed);
        assert_eq!(state.current_tab, 1);
        assert_eq!(state.selected, 4);

        state.handle_key(KeyCode::Left, KeyModifiers::NONE, 20, 48, &running, &closed);
        assert_eq!(state.current_tab, 0);
        assert_eq!(state.selected, 0);

        state.handle_key(KeyCode::Left, KeyModifiers::NONE, 20, 48, &running, &closed);
        assert_eq!(state.current_tab, 0);
    }

    #[test]
    fn test_tab_auto_switch_on_cycle() {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let closed = Arc::new(Mutex::new(HashSet::new()));
        let mut state = make_state(&["a", "b", "c", "d", "e", "f", "g", "h"], 0);
        state.selected = 3;

        state.handle_key(KeyCode::Tab, KeyModifiers::NONE, 20, 48, &running, &closed);
        assert_eq!(state.selected, 4);
        assert_eq!(state.current_tab, 1);
    }

    #[test]
    fn test_few_panes_no_tabs() {
        let state = make_state(&["a", "b"], 0);
        assert_eq!(state.total_tabs(48), 1);
        assert_eq!(state.visible_indices(48), vec![0, 1]);
    }
}
