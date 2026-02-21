mod simple;
mod tui;

use anyhow::{Context, Result};
use regex::Regex;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::utils::spinner;

#[derive(Clone)]
pub struct PodInfo {
    pub namespace: String,
    pub name: String,
    pub pattern_idx: usize,
}

impl PodInfo {
    pub fn key(&self) -> String {
        format!("{}/{}", self.namespace, self.name)
    }
}

pub fn run(patterns: &[String], err_only: bool, simple: bool) -> Result<()> {
    let regexes: Vec<Regex> = patterns.iter().map(|p| pod_pattern_regex(p)).collect();

    let mut sp = spinner::create_spinner("Finding pods...");

    let pods = match find_matching_pods(&regexes) {
        Ok(p) => p,
        Err(e) => {
            spinner::stop_spinner(sp.as_mut());
            return Err(e);
        }
    };

    spinner::finish_with_message(sp.as_mut(), "Found pods");

    let use_color = atty::is(atty::Stream::Stdout);

    let mut any_match = false;
    let mut has_warnings = false;
    for (i, pattern) in patterns.iter().enumerate() {
        let has_match = pods.iter().any(|p| p.pattern_idx == i);
        if has_match {
            any_match = true;
        } else {
            print_no_match_warning(pattern, use_color);
            has_warnings = true;
        }
    }

    if !any_match {
        return Ok(());
    }

    if has_warnings && !simple {
        thread::sleep(Duration::from_secs(2));
    }

    if simple {
        simple::run(pods, regexes, err_only)
    } else {
        tui::run(pods, regexes, err_only)
    }
}

const KUBECTL_AUTH_TIMEOUT: Duration = Duration::from_secs(15);

pub fn find_matching_pods(regexes: &[Regex]) -> Result<Vec<PodInfo>> {
    let mut child = Command::new("kubectl")
        .args([
            "get",
            "pods",
            "-A",
            "-o",
            "custom-columns=NAMESPACE:.metadata.namespace,NAME:.metadata.name",
            "--no-headers",
            "--request-timeout=10s",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to run kubectl get pods")?;

    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut out = Vec::new();
        let _ = stdout.read_to_end(&mut out);
        let mut err = Vec::new();
        let _ = stderr.read_to_end(&mut err);
        let _ = tx.send((out, err));
    });

    let deadline = Instant::now() + KUBECTL_AUTH_TIMEOUT;
    let status = loop {
        match child.try_wait()? {
            Some(s) => break s,
            None => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    let stderr_msg = rx
                        .recv_timeout(Duration::from_secs(1))
                        .ok()
                        .and_then(|(_, e)| String::from_utf8(e).ok())
                        .filter(|s| !s.trim().is_empty())
                        .map(|s| format!("\n\nkubectl stderr:\n{s}"))
                        .unwrap_or_default();
                    anyhow::bail!(
                        "kubectl get pods timed out ({}s). \
                         If your cluster requires authentication, run your auth command first \
                         (e.g. open the login URL in a browser or run the token command), then run track again.{}",
                        KUBECTL_AUTH_TIMEOUT.as_secs(),
                        stderr_msg
                    );
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    };

    let (stdout_bytes, stderr_bytes) = rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap_or_else(|_| (Vec::new(), Vec::new()));

    if !status.success() {
        let stderr_str = String::from_utf8_lossy(&stderr_bytes);
        let hint = if stderr_str.contains("could not open the browser")
            || stderr_str.contains("Please visit the following URL")
            || stderr_str.contains("authenticate")
        {
            " Authenticate to your cluster first (e.g. open the login URL in a browser or run your auth command), then run track again."
        } else {
            ""
        };
        anyhow::bail!(
            "kubectl get pods failed{}.{}",
            hint,
            if stderr_str.trim().is_empty() {
                String::new()
            } else {
                format!("\n\nkubectl stderr:\n{stderr_str}")
            }
        );
    }

    let stdout = String::from_utf8(stdout_bytes)?;
    let mut pods = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let namespace = parts[0];
        let pod_name = parts[1];

        for (i, re) in regexes.iter().enumerate() {
            if re.is_match(pod_name) {
                pods.push(PodInfo {
                    namespace: namespace.to_string(),
                    name: pod_name.to_string(),
                    pattern_idx: i,
                });
                break;
            }
        }
    }

    Ok(pods)
}

pub fn should_show_line(line: &str, in_traceback: &mut bool) -> bool {
    if line.contains("Traceback (most recent call last)") {
        *in_traceback = true;
        return true;
    }

    if *in_traceback {
        if line.starts_with(' ') || line.starts_with('\t') {
            return true;
        }
        *in_traceback = false;
        if !line.is_empty() {
            return true;
        }
    }

    let upper = line.to_uppercase();
    upper.contains("WARNING")
        || upper.contains("ERROR")
        || upper.contains("CRITICAL")
        || upper.contains("FATAL")
}

pub fn pod_pattern_regex(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|_| {
        let escaped = regex::escape(pattern);
        Regex::new(&format!("(?i){}", escaped)).expect("escaped pattern must be valid")
    })
}

fn print_no_match_warning(pattern: &str, use_color: bool) {
    if use_color {
        eprintln!(
            "\n\x1b[93m\x1b[1m\u{26a0}  No pods found matching pattern: \"{pattern}\"\x1b[0m\n"
        );
    } else {
        eprintln!("\nNo pods found matching pattern: \"{pattern}\"\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_info_key() {
        let cases = vec![
            ("default", "my-pod", "default/my-pod"),
            ("kube-system", "coredns-abc123", "kube-system/coredns-abc123"),
            ("ns", "a", "ns/a"),
        ];
        for (ns, name, expected) in cases {
            let pod = PodInfo {
                namespace: ns.to_string(),
                name: name.to_string(),
                pattern_idx: 0,
            };
            assert_eq!(pod.key(), expected);
        }
    }

    #[test]
    fn test_pod_pattern_regex_valid() {
        let re = pod_pattern_regex("api-.*");
        assert!(re.is_match("api-server"));
        assert!(re.is_match("api-worker-123"));
        assert!(!re.is_match("frontend"));
    }

    #[test]
    fn test_pod_pattern_regex_invalid_falls_back_case_insensitive() {
        let re = pod_pattern_regex("[invalid");
        assert!(re.is_match("[invalid"));
        assert!(re.is_match("[INVALID"));
        assert!(re.is_match("[Invalid"));
    }

    #[test]
    fn test_pod_pattern_regex_case_sensitive_by_default() {
        let re = pod_pattern_regex("MyPod");
        assert!(re.is_match("MyPod"));
        assert!(!re.is_match("mypod"));
    }

    #[test]
    fn test_should_show_line_error_keywords() {
        let cases = vec![
            ("2026-01-01 ERROR something broke", true),
            ("2026-01-01 WARNING disk full", true),
            ("2026-01-01 CRITICAL out of memory", true),
            ("2026-01-01 FATAL crash", true),
            ("2026-01-01 error lowercase", true),
            ("2026-01-01 warning lowercase", true),
            ("2026-01-01 Info normal log", false),
            ("2026-01-01 DEBUG verbose", false),
            ("just a normal line", false),
            ("", false),
        ];
        for (line, expected) in cases {
            let mut in_traceback = false;
            assert_eq!(
                should_show_line(line, &mut in_traceback),
                expected,
                "Failed for line: {line:?}"
            );
        }
    }

    #[test]
    fn test_should_show_line_traceback_sequence() {
        let mut in_traceback = false;

        assert!(should_show_line(
            "Traceback (most recent call last):",
            &mut in_traceback
        ));
        assert!(in_traceback);

        assert!(should_show_line(
            "  File \"main.py\", line 10, in <module>",
            &mut in_traceback
        ));
        assert!(in_traceback);

        assert!(should_show_line(
            "    result = do_thing()",
            &mut in_traceback
        ));
        assert!(in_traceback);

        assert!(should_show_line("ValueError: bad value", &mut in_traceback));
        assert!(!in_traceback);

        assert!(!should_show_line("normal log after traceback", &mut in_traceback));
    }

    #[test]
    fn test_should_show_line_traceback_with_tabs() {
        let mut in_traceback = false;

        assert!(should_show_line(
            "Traceback (most recent call last):",
            &mut in_traceback
        ));
        assert!(should_show_line(
            "\tFile \"main.py\", line 5",
            &mut in_traceback
        ));
        assert!(in_traceback);
    }

    #[test]
    fn test_should_show_line_no_traceback_state_leak() {
        let mut in_traceback = false;

        assert!(!should_show_line("INFO all good", &mut in_traceback));
        assert!(!in_traceback);

        assert!(!should_show_line("DEBUG details", &mut in_traceback));
        assert!(!in_traceback);
    }
}
