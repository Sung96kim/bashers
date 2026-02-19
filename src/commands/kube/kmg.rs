use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use regex::Regex;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const CYAN_BOLD: &str = "\x1b[36m\x1b[1m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

fn format_pod_prefix(pod_name: &str, use_color: bool) -> String {
    if use_color {
        format!("{CYAN_BOLD}[{pod_name}]{RESET}: ")
    } else {
        format!("[{pod_name}]: ")
    }
}

pub fn run(pattern: &str) -> Result<()> {
    let use_color = atty::is(atty::Stream::Stderr);

    let draw_target = if atty::is(atty::Stream::Stderr) {
        ProgressDrawTarget::stderr()
    } else {
        ProgressDrawTarget::hidden()
    };
    let multi = MultiProgress::with_draw_target(draw_target);

    let header_style = ProgressStyle::default_spinner()
        .template("{spinner:.dim}{msg}")
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", ""]);
    let header_pb = multi.add(
        ProgressBar::new_spinner()
            .with_style(header_style)
            .with_message(" Fetching pods..."),
    );
    header_pb.enable_steady_tick(Duration::from_millis(80));

    let pods_output = Command::new("kubectl")
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

    if !pods_output.status.success() {
        let msg = if use_color {
            format!("{RED}✗ kubectl get pods failed{RESET}")
        } else {
            "✗ kubectl get pods failed".to_string()
        };
        header_pb.finish_with_message(msg);
        anyhow::bail!("kubectl get pods failed");
    }

    let fetched_msg = if use_color {
        format!("{GREEN}✓ Fetched pods{RESET}")
    } else {
        "✓ Fetched pods".to_string()
    };
    header_pb.finish_with_message(fetched_msg);

    let stdout = String::from_utf8(pods_output.stdout)?;
    let re = pod_pattern_regex(pattern);
    let pods: Vec<(String, String)> = stdout
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return None;
            }
            let namespace = parts[0];
            let pod_name = parts[1];
            if !re.is_match(pod_name) {
                return None;
            }
            Some((namespace.to_string(), pod_name.to_string()))
        })
        .collect();

    let pod_style = ProgressStyle::default_spinner()
        .template("{prefix}{spinner:.dim}{msg}")
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", ""]);

    let total = pods.len();
    let mut handles = Vec::with_capacity(total);

    for (idx, (namespace, pod_name)) in pods.into_iter().enumerate() {
        let step = format!("[{}/{}] ", idx + 1, total);
        let prefix = format!("{}{}", step, format_pod_prefix(&pod_name, use_color));
        let pb = ProgressBar::new_spinner()
            .with_style(pod_style.clone())
            .with_prefix(prefix)
            .with_message("");
        let pb = multi.add(pb);
        pb.enable_steady_tick(Duration::from_millis(80));

        let idx = idx + 1;
        let handle = thread::spawn(move || {
            let describe_output = Command::new("kubectl")
                .args(["describe", "pod", &pod_name, "-n", &namespace])
                .stdout(Stdio::piped())
                .output();

            let image = match describe_output {
                Ok(ref out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .find_map(|line| {
                        line.trim()
                            .strip_prefix("Image:")
                            .map(|s| s.trim().to_string())
                    })
                    .unwrap_or_default(),
                _ => String::new(),
            };

            let msg = if image.is_empty() {
                "(no image)".to_string()
            } else {
                image
            };
            let step = format!("[{}/{}] ", idx, total);
            pb.set_prefix(format!(
                "{}{}",
                step,
                format_pod_prefix(&pod_name, use_color)
            ));
            pb.finish_with_message(msg);
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.join();
    }

    Ok(())
}

fn pod_pattern_regex(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|_| {
        let escaped = regex::escape(pattern);
        Regex::new(&format!("(?i){}", escaped)).expect("escaped pattern must be valid")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_pattern_regex_valid() {
        let re = pod_pattern_regex("my-pod");
        assert!(re.is_match("my-pod"));
        assert!(!re.is_match("other"));
    }

    #[test]
    fn test_pod_pattern_regex_invalid_falls_back_case_insensitive() {
        let re = pod_pattern_regex("[invalid");
        assert!(re.is_match("[invalid"));
        assert!(re.is_match("[INVALID"));
    }

    #[test]
    fn test_pod_pattern_regex_literal_bracket_escaped_on_fallback() {
        let re = pod_pattern_regex("[");
        assert!(re.is_match("["));
    }
}
