mod simple;
mod tui;

use anyhow::{Context, Result};
use regex::Regex;
use std::process::Command;
use std::thread;
use std::time::Duration;

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
    let pods = find_matching_pods(&regexes)?;
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

pub fn find_matching_pods(regexes: &[Regex]) -> Result<Vec<PodInfo>> {
    let output = Command::new("kubectl")
        .args([
            "get",
            "pods",
            "-A",
            "-o",
            "custom-columns=NAMESPACE:.metadata.namespace,NAME:.metadata.name",
            "--no-headers",
        ])
        .output()
        .context("Failed to run kubectl get pods")?;

    if !output.status.success() {
        anyhow::bail!("kubectl get pods failed");
    }

    let stdout = String::from_utf8(output.stdout)?;
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
