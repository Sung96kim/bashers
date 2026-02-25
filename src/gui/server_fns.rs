use dioxus::prelude::*;

use crate::commands::kube::track::PodInfo;
use crate::commands::show::DependencyInfo;
use crate::utils::project::ProjectType;

#[server]
pub async fn search_pods(patterns: Vec<String>) -> Result<Vec<PodInfo>, ServerFnError> {
    use crate::commands::kube::pod_pattern_regex;
    use crate::commands::kube::track::find_matching_pods;

    let regexes: Vec<regex::Regex> = patterns.iter().map(|p| pod_pattern_regex(p)).collect();
    let pods = find_matching_pods(&regexes).map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(pods)
}

#[server]
pub async fn get_pod_logs(
    namespace: String,
    name: String,
    tail: u64,
) -> Result<Vec<String>, ServerFnError> {
    use std::process::{Command, Stdio};

    let output = Command::new("kubectl")
        .args([
            "logs",
            "--tail",
            &tail.to_string(),
            "-n",
            &namespace,
            &name,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| ServerFnError::new(format!("kubectl logs failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ServerFnError::new(format!("kubectl logs error: {stderr}")));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = stdout.lines().map(String::from).collect();
    Ok(lines)
}

#[server]
pub async fn list_dependencies() -> Result<(ProjectType, Vec<DependencyInfo>), ServerFnError> {
    use crate::commands::show::{get_dependency_output, parse_dependency_lines};

    let (pt, lines) =
        get_dependency_output(&[]).map_err(|e| ServerFnError::new(e.to_string()))?;
    let deps = parse_dependency_lines(&lines);
    Ok((pt, deps))
}

#[server]
pub async fn list_packages() -> Result<(ProjectType, Vec<String>), ServerFnError> {
    use crate::utils::{packages, project};

    let pt = project::detect()
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No project detected".to_string()))?;
    let pkgs = packages::list(pt).map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok((pt, pkgs))
}

#[server]
pub async fn run_command(
    program: String,
    args: Vec<String>,
) -> Result<String, ServerFnError> {
    use crate::commands::watch::run_cmd;

    let output = run_cmd(&program, &args).map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(output)
}
