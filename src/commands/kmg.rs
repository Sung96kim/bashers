use anyhow::{Context, Result};
use regex::Regex;
use std::process::{Command, Stdio};

pub fn run(pattern: &str) -> Result<()> {
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
        anyhow::bail!("kubectl get pods failed");
    }

    let stdout = String::from_utf8(pods_output.stdout)?;
    let re = pod_pattern_regex(pattern);

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
            if describe_line.contains("Image:") {
                println!("{}", describe_line.trim());
            }
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
