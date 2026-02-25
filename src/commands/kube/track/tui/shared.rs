use regex::Regex;
use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};

use super::event::TrackEvent;

pub struct SharedState {
    pub err_only: bool,
    pub running: Arc<AtomicBool>,
    pub active_pods: Arc<Mutex<HashSet<String>>>,
    pub closed_pods: Arc<Mutex<HashSet<String>>>,
    pub regexes: Arc<Mutex<Vec<Regex>>>,
    pub tx: mpsc::Sender<TrackEvent>,
}

impl SharedState {
    pub fn new(
        err_only: bool,
        initial_regexes: Vec<Regex>,
    ) -> (Self, mpsc::Receiver<TrackEvent>) {
        let (tx, rx) = mpsc::channel();
        let shared = Self {
            err_only,
            running: Arc::new(AtomicBool::new(true)),
            active_pods: Arc::new(Mutex::new(HashSet::new())),
            closed_pods: Arc::new(Mutex::new(HashSet::new())),
            regexes: Arc::new(Mutex::new(initial_regexes)),
            tx: tx.clone(),
        };
        (shared, rx)
    }

    pub fn add_regex(&self, regex: Regex) {
        self.regexes.lock().unwrap().push(regex);
    }

    pub fn clone_regexes(&self) -> Vec<Regex> {
        self.regexes.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_new_creates_valid_state() {
        let regexes = vec![Regex::new("test-.*").unwrap()];
        let (state, _rx) = SharedState::new(false, regexes);
        assert!(!state.err_only);
        assert!(state.running.load(Ordering::SeqCst));
        assert!(state.active_pods.lock().unwrap().is_empty());
        assert!(state.closed_pods.lock().unwrap().is_empty());
        assert_eq!(state.clone_regexes().len(), 1);
    }

    #[test]
    fn test_new_with_err_only() {
        let (state, _rx) = SharedState::new(true, vec![]);
        assert!(state.err_only);
    }

    #[test]
    fn test_add_regex() {
        let (state, _rx) = SharedState::new(false, vec![]);
        assert_eq!(state.clone_regexes().len(), 0);

        state.add_regex(Regex::new("pod-a").unwrap());
        assert_eq!(state.clone_regexes().len(), 1);

        state.add_regex(Regex::new("pod-b").unwrap());
        assert_eq!(state.clone_regexes().len(), 2);
    }

    #[test]
    fn test_clone_regexes_returns_copy() {
        let regexes = vec![Regex::new("test").unwrap()];
        let (state, _rx) = SharedState::new(false, regexes);

        let cloned = state.clone_regexes();
        assert_eq!(cloned.len(), 1);
        assert!(cloned[0].is_match("test"));
    }

    #[test]
    fn test_tx_can_send_events() {
        let (state, rx) = SharedState::new(false, vec![]);
        state
            .tx
            .send(TrackEvent::LogLine {
                pod_key: "ns/pod".to_string(),
                text: "hello".to_string(),
            })
            .unwrap();

        match rx.recv().unwrap() {
            TrackEvent::LogLine { pod_key, text } => {
                assert_eq!(pod_key, "ns/pod");
                assert_eq!(text, "hello");
            }
            _ => panic!("unexpected event type"),
        }
    }

    #[test]
    fn test_running_flag_toggleable() {
        let (state, _rx) = SharedState::new(false, vec![]);
        assert!(state.running.load(Ordering::SeqCst));
        state.running.store(false, Ordering::SeqCst);
        assert!(!state.running.load(Ordering::SeqCst));
    }
}
