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
