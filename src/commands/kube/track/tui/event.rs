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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_line_clone() {
        let event = TrackEvent::LogLine {
            pod_key: "ns/pod".to_string(),
            text: "log message".to_string(),
        };
        let cloned = event.clone();
        match cloned {
            TrackEvent::LogLine { pod_key, text } => {
                assert_eq!(pod_key, "ns/pod");
                assert_eq!(text, "log message");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_new_pod_clone() {
        let pod = PodInfo {
            namespace: "default".to_string(),
            name: "my-pod".to_string(),
            pattern_idx: 0,
        };
        let alive = Arc::new(AtomicBool::new(true));
        let event = TrackEvent::NewPod {
            pod: pod.clone(),
            alive: alive.clone(),
        };
        let cloned = event.clone();
        match cloned {
            TrackEvent::NewPod { pod, alive } => {
                assert_eq!(pod.namespace, "default");
                assert_eq!(pod.name, "my-pod");
                assert!(alive.load(std::sync::atomic::Ordering::SeqCst));
            }
            _ => panic!("wrong variant"),
        }
    }
}
