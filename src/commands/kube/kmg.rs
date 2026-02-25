use crate::utils::colors;
use crate::utils::multi_progress;
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::BTreeMap;
use std::process::{Command, Stdio};

use super::pod_pattern_regex;

fn format_pod_prefix(pod_name: &str, use_color: bool) -> String {
    if use_color {
        format!(
            "{}[{pod_name}]{}: ",
            colors::ANSI_CYAN_BOLD,
            colors::ANSI_RESET
        )
    } else {
        format!("[{pod_name}]: ")
    }
}

pub fn run(patterns: &[String]) -> Result<()> {
    let use_color = atty::is(atty::Stream::Stderr);
    let multi = multi_progress::multi_progress_stderr();
    let patterns_display = patterns.join(" ");

    let success_msg = if use_color {
        format!(
            "{}✓ Fetched pods matching patterns: {patterns_display}{}",
            colors::ANSI_GREEN,
            colors::ANSI_RESET
        )
    } else {
        format!("✓ Fetched pods matching patterns: {patterns_display}")
    };
    let failure_msg = if use_color {
        format!(
            "{}✗ kubectl get pods failed{}",
            colors::ANSI_RED,
            colors::ANSI_RESET
        )
    } else {
        "✗ kubectl get pods failed".to_string()
    };

    let loading_msg = format!(" Fetching pods matching patterns: {patterns_display}...");
    let pods_output =
        multi_progress::run_header_spinner(&multi, &loading_msg, success_msg, failure_msg, || {
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
            Ok(output)
        })?;

    let stdout = String::from_utf8(pods_output.stdout)?;
    let regexes: Vec<Regex> = patterns
        .iter()
        .map(|p| pod_pattern_regex(p.as_str()))
        .collect();
    let pods_with_pattern: Vec<(String, String, usize)> = stdout
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
            let pattern_idx = regexes.iter().position(|re| re.is_match(pod_name))?;
            Some((namespace.to_string(), pod_name.to_string(), pattern_idx))
        })
        .collect();

    let by_pattern: BTreeMap<usize, Vec<(String, String)>> =
        pods_with_pattern
            .into_iter()
            .fold(BTreeMap::new(), |mut acc, (ns, name, idx)| {
                acc.entry(idx).or_default().push((ns, name));
                acc
            });

    let sections: Vec<(String, Vec<(String, String)>)> = by_pattern
        .into_iter()
        .map(|(pattern_idx, pods)| (patterns[pattern_idx].clone(), pods))
        .collect();

    let _ = multi_progress::run_parallel_spinners_sectioned(
        &multi,
        sections,
        |_section_idx, one_indexed, total_in_section, (_, pod_name)| {
            format!(
                "[{}/{}] {}",
                one_indexed,
                total_in_section,
                format_pod_prefix(pod_name, use_color)
            )
        },
        |(namespace, pod_name): (String, String)| {
            let describe_output = Command::new("kubectl")
                .args(["describe", "pod", &pod_name, "-n", &namespace])
                .stdout(Stdio::piped())
                .output();

            match describe_output {
                Ok(ref out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .find_map(|line| {
                        line.trim()
                            .strip_prefix("Image:")
                            .map(|s| s.trim().to_string())
                    })
                    .unwrap_or_default(),
                _ => String::new(),
            }
        },
        |image: &String| {
            if image.is_empty() {
                "(no image)".to_string()
            } else {
                image.clone()
            }
        },
    );

    Ok(())
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

    #[test]
    fn test_format_pod_prefix_with_color() {
        let result = format_pod_prefix("api-server-abc123", true);
        assert!(result.contains("api-server-abc123"));
        assert!(result.contains(colors::ANSI_CYAN_BOLD));
        assert!(result.contains(colors::ANSI_RESET));
    }

    #[test]
    fn test_format_pod_prefix_without_color() {
        let result = format_pod_prefix("api-server-abc123", false);
        assert_eq!(result, "[api-server-abc123]: ");
        assert!(!result.contains('\x1b'));
    }

    #[test]
    fn test_format_pod_prefix_empty_name() {
        let result = format_pod_prefix("", false);
        assert_eq!(result, "[]: ");
    }
}
