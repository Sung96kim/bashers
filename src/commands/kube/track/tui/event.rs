use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use super::super::PodInfo;

#[derive(Clone)]
pub enum TrackEvent {
    LogLine {
        pod_key: String,
        text: String,
    },
    NewPod {
        pod: PodInfo,
        alive: Arc<AtomicBool>,
    },
}
