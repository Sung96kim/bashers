use anyhow::{Context, Result};
use regex::Regex;
use std::process::{Command, Stdio};

use crate::utils::spinner;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";

pub fn run(pattern: &str) -> Result<()> {
    let use_color = atty::is(atty::Stream::Stdout);
    let mut sp = spinner::create_spinner("Getting pod images...");

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
        spinner::stop_spinner(sp.as_mut());
        anyhow::bail!("kubectl get pods failed");
    }

    let stdout = String::from_utf8(pods_output.stdout)?;
    let re = pod_pattern_regex(pattern);
    let mut results: Vec<(String, String)> = Vec::new();

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
        if !re.is_match(pod_name) {
            continue;
        }

        let describe_output = Command::new("kubectl")
            .args(["describe", "pod", pod_name, "-n", namespace])
            .stdout(Stdio::piped())
            .output()
            .context("Failed to run kubectl describe pod")?;

        if !describe_output.status.success() {
            continue;
        }

        let describe_stdout = String::from_utf8(describe_output.stdout)?;
        for describe_line in describe_stdout.lines() {
            if let Some(image) = describe_line.trim().strip_prefix("Image:") {
                results.push((pod_name.to_string(), image.trim().to_string()));
            }
        }
    }

    spinner::finish_with_message(sp.as_mut(), &format!("Retrieved images for {pattern}"));

    for (pod_name, image) in results {
        if use_color {
            println!("{CYAN}{BOLD}[{pod_name}]{RESET}: {image}");
        } else {
            println!("[{pod_name}]: {image}");
        }
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
