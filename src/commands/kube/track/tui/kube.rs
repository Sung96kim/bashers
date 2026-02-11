use anyhow::Result;
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::event::TrackEvent;
use super::super::{
    find_matching_pods, pod_pattern_regex, should_show_line, PodInfo,
};
use super::traits::{LogStreamSpawnOpts, LogStreamSpawner, PodDiscovery, PatternToRegex};

fn should_stop(running: &Arc<AtomicBool>, alive: &Arc<AtomicBool>) -> bool {
    !running.load(Ordering::SeqCst) || !alive.load(Ordering::SeqCst)
}

pub struct KubePodDiscovery;

impl PodDiscovery for KubePodDiscovery {
    fn find_matching_pods(&self, regexes: &[Regex]) -> Result<Vec<PodInfo>> {
        find_matching_pods(regexes)
    }
}

pub struct KubePatternToRegex;

impl PatternToRegex for KubePatternToRegex {
    fn build(&self, pattern: &str) -> Regex {
        pod_pattern_regex(pattern)
    }
}

pub struct KubectlLogSpawner;

impl LogStreamSpawner for KubectlLogSpawner {
    fn spawn(&self, pod: &PodInfo, opts: LogStreamSpawnOpts) {
        let ns = pod.namespace.clone();
        let name = pod.name.clone();
        let key = pod.key();
        let err_only = opts.err_only;
        let running = opts.running;
        let alive = opts.alive;
        let active_pods = opts.active_pods;
        let tx = opts.tx;

        thread::spawn(move || {
            loop {
                if should_stop(&running, &alive) {
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
                                if should_stop(&running, &alive) {
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

                if should_stop(&running, &alive) {
                    break;
                }

                thread::sleep(Duration::from_secs(3));
            }

            active_pods.lock().unwrap().remove(&key);
        });
    }
}
