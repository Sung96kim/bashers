use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};

use super::event::TrackEvent;
use super::super::PodInfo;

pub trait PodDiscovery: Send + Sync {
    fn find_matching_pods(&self, regexes: &[Regex]) -> Result<Vec<PodInfo>>;
}

pub struct LogStreamSpawnOpts {
    pub err_only: bool,
    pub running: Arc<AtomicBool>,
    pub alive: Arc<AtomicBool>,
    pub active_pods: Arc<Mutex<HashSet<String>>>,
    pub tx: mpsc::Sender<TrackEvent>,
}

pub trait LogStreamSpawner: Send + Sync {
    fn spawn(&self, pod: &PodInfo, opts: LogStreamSpawnOpts);
}

pub trait PatternToRegex: Send + Sync {
    fn build(&self, pattern: &str) -> Regex;
}
