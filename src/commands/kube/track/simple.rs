use super::{find_matching_pods, should_show_line, PodInfo};
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";

const POD_COLORS: &[&str] = &[
    "\x1b[36m",
    "\x1b[32m",
    "\x1b[35m",
    "\x1b[33m",
    "\x1b[34m",
    "\x1b[96m",
    "\x1b[92m",
    "\x1b[95m",
];

struct OutputState {
    last_pod: String,
    use_color: bool,
}

pub fn run(pods: Vec<PodInfo>, regexes: Vec<Regex>, err_only: bool) -> Result<()> {
    let use_color = atty::is(atty::Stream::Stdout);
    let running = Arc::new(AtomicBool::new(true));

    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl+C handler")?;

    let active_pods: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let output_state = Arc::new(Mutex::new(OutputState {
        last_pod: String::new(),
        use_color,
    }));

    for pod in &pods {
        active_pods.lock().unwrap().insert(pod.key());
        spawn_log_follower(
            &pod.namespace,
            &pod.name,
            pod.pattern_idx,
            err_only,
            running.clone(),
            active_pods.clone(),
            output_state.clone(),
        );
    }

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(5));
        if !running.load(Ordering::SeqCst) {
            break;
        }

        if let Ok(new_pods) = find_matching_pods(&regexes) {
            for pod in &new_pods {
                let key = pod.key();
                let should_spawn = {
                    let mut active = active_pods.lock().unwrap();
                    if active.contains(&key) {
                        false
                    } else {
                        active.insert(key);
                        true
                    }
                };

                if should_spawn {
                    spawn_log_follower(
                        &pod.namespace,
                        &pod.name,
                        pod.pattern_idx,
                        err_only,
                        running.clone(),
                        active_pods.clone(),
                        output_state.clone(),
                    );
                }
            }
        }
    }

    Ok(())
}

fn spawn_log_follower(
    namespace: &str,
    pod_name: &str,
    pattern_idx: usize,
    err_only: bool,
    running: Arc<AtomicBool>,
    active_pods: Arc<Mutex<HashSet<String>>>,
    output_state: Arc<Mutex<OutputState>>,
) {
    let ns = namespace.to_string();
    let name = pod_name.to_string();
    let color = POD_COLORS[pattern_idx % POD_COLORS.len()];

    thread::spawn(move || {
        let key = format!("{}/{}", ns, name);

        loop {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            let result = Command::new("kubectl")
                .args(["logs", "-f", "--tail=1000", "--timestamps", &name, "-n", &ns])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match result {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        let mut in_traceback = false;

                        for line in reader.lines() {
                            if !running.load(Ordering::SeqCst) {
                                let _ = child.kill();
                                break;
                            }

                            match line {
                                Ok(text) => {
                                    if err_only && !should_show_line(&text, &mut in_traceback) {
                                        continue;
                                    }
                                    let mut state = output_state.lock().unwrap();
                                    if state.last_pod != key {
                                        let separator = "\u{2501}".repeat(40);
                                        if state.use_color {
                                            println!(
                                                "\n{color}{BOLD}{separator}{RESET}\n{color}{BOLD} {key}{RESET}\n{color}{BOLD}{separator}{RESET}"
                                            );
                                        } else {
                                            println!("\n{separator}\n {key}\n{separator}");
                                        }
                                        state.last_pod = key.clone();
                                    }
                                    println!("{text}");
                                }
                                Err(_) => break,
                            }
                        }
                    }
                    let _ = child.wait();
                }
                Err(e) => {
                    eprintln!(
                        "\n{RED}{BOLD}[error]{RESET} Failed to follow logs for {key}: {e}\n"
                    );
                    break;
                }
            }

            if !running.load(Ordering::SeqCst) {
                break;
            }

            thread::sleep(Duration::from_secs(3));
        }

        active_pods.lock().unwrap().remove(&key);
    });
}
