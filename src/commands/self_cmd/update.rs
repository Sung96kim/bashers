use crate::utils::{colors::Colors, spinner};
use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const GITHUB_REPO: &str = "Sung96kim/bashers";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run() -> Result<()> {
    let mut colors = Colors::new();
    
    let latest_version = if spinner::should_show_spinner() {
        let pb = spinner::create_spinner();
        pb.set_message("Checking for updates...".to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        let result = get_latest_version();
        pb.finish_and_clear();
        result?
    } else {
        colors.green()?;
        colors.print("Checking for updates...")?;
        colors.reset()?;
        colors.println("")?;
        get_latest_version()?
    };
    
    if latest_version == CURRENT_VERSION {
        colors.green()?;
        colors.println(&format!("Already up to date (v{})", CURRENT_VERSION))?;
        colors.reset()?;
        return Ok(());
    }

    colors.green()?;
    colors.print(&format!("New version available: v{}", latest_version))?;
    colors.reset()?;
    colors.println("")?;
    colors.print(&format!("Current version: v{}", CURRENT_VERSION))?;
    colors.println("")?;

    let binary_path = get_binary_path()?;
    let temp_path = binary_path.with_extension("tmp");

    if spinner::should_show_spinner() {
        let pb = spinner::create_spinner();
        pb.set_message("Downloading latest version...".to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        let result = download_binary(&latest_version, &temp_path);
        pb.finish_and_clear();
        result?;
    } else {
        colors.green()?;
        colors.print("Downloading latest version...")?;
        colors.reset()?;
        colors.println("")?;
        download_binary(&latest_version, &temp_path)?;
    }

    if spinner::should_show_spinner() {
        let pb = spinner::create_spinner();
        pb.set_message("Installing update...".to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        
        fs::rename(&temp_path, &binary_path)
            .context("Failed to replace binary")?;
        
        chmod_executable(&binary_path)?;
        
        pb.finish_and_clear();
    } else {
        colors.green()?;
        colors.print("Installing update...")?;
        colors.reset()?;
        colors.println("")?;

        fs::rename(&temp_path, &binary_path)
            .context("Failed to replace binary")?;

        chmod_executable(&binary_path)?;
    }

    colors.green()?;
    colors.println(&format!("Successfully updated to v{}", latest_version))?;
    colors.reset()?;

    Ok(())
}

fn get_latest_version() -> Result<String> {
    let output = Command::new("curl")
        .args([
            "-s",
            "-H",
            "Accept: application/vnd.github.v3+json",
            &format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO),
        ])
        .output()
        .context("Failed to fetch latest version from GitHub")?;

    if !output.status.success() {
        anyhow::bail!("Failed to fetch latest version");
    }

    let response = String::from_utf8(output.stdout)?;
    
    let re = Regex::new(r#""tag_name"\s*:\s*"v?([^"]+)""#)
        .context("Failed to create regex")?;
    
    let version = re
        .captures(&response)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .context("No tag_name in GitHub API response")?;

    Ok(version)
}

fn download_binary(version: &str, output_path: &PathBuf) -> Result<()> {
    let url = format!(
        "https://github.com/{}/releases/download/v{}/bashers-linux-x86_64.tar.gz",
        GITHUB_REPO, version
    );

    let curl_output = Command::new("curl")
        .args(["-sL", &url])
        .output()
        .context("Failed to download binary")?;

    if !curl_output.status.success() {
        anyhow::bail!("Failed to download binary from GitHub releases");
    }

    let mut tar_process = Command::new("tar")
        .args(["-xzf", "-", "-C", "/tmp"])
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn tar process")?;

    if let Some(mut stdin) = tar_process.stdin.take() {
        use std::io::Write;
        stdin.write_all(&curl_output.stdout)?;
        stdin.flush()?;
    }

    let tar_output = tar_process
        .wait_with_output()
        .context("Failed to extract binary")?;

    if !tar_output.status.success() {
        anyhow::bail!("Failed to extract binary archive");
    }

    let extracted_binary = PathBuf::from("/tmp/bashers");
    if !extracted_binary.exists() {
        anyhow::bail!("Extracted binary not found");
    }

    fs::copy(&extracted_binary, output_path)
        .context("Failed to copy binary to target location")?;

    fs::remove_file(&extracted_binary).ok();

    Ok(())
}

fn get_binary_path() -> Result<PathBuf> {
    let current_exe = std::env::current_exe()
        .context("Failed to get current executable path")?;

    Ok(current_exe)
}

fn chmod_executable(path: &PathBuf) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_from_github_api() {
        // Test parsing version from GitHub API response
        let response = r#"{"tag_name":"v0.4.9","name":"v0.4.9"}"#;
        let re = Regex::new(r#""tag_name"\s*:\s*"v?([^"]+)""#).unwrap();
        let version = re
            .captures(response)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());
        assert_eq!(version, Some("0.4.9".to_string()));
    }

    #[test]
    fn test_parse_version_without_v_prefix() {
        // Test parsing version without v prefix
        let response = r#"{"tag_name":"0.4.9","name":"0.4.9"}"#;
        let re = Regex::new(r#""tag_name"\s*:\s*"v?([^"]+)""#).unwrap();
        let version = re
            .captures(response)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());
        assert_eq!(version, Some("0.4.9".to_string()));
    }

    #[test]
    fn test_parse_version_with_whitespace() {
        // Test parsing version with whitespace in JSON
        let response = r#"{"tag_name" : "v0.4.9"}"#;
        let re = Regex::new(r#""tag_name"\s*:\s*"v?([^"]+)""#).unwrap();
        let version = re
            .captures(response)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());
        assert_eq!(version, Some("0.4.9".to_string()));
    }

    #[test]
    fn test_parse_version_multiline_response() {
        // Test parsing version from multiline JSON response
        let response = r#"{
            "tag_name": "v0.4.9",
            "name": "Release v0.4.9"
        }"#;
        let re = Regex::new(r#""tag_name"\s*:\s*"v?([^"]+)""#).unwrap();
        let version = re
            .captures(response)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());
        assert_eq!(version, Some("0.4.9".to_string()));
    }

    #[test]
    fn test_parse_version_invalid_response() {
        // Test parsing invalid response (no tag_name)
        let response = r#"{"name":"v0.4.9"}"#;
        let re = Regex::new(r#""tag_name"\s*:\s*"v?([^"]+)""#).unwrap();
        let version = re
            .captures(response)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());
        assert_eq!(version, None);
    }

    #[test]
    fn test_get_binary_path() {
        // Test that get_binary_path returns a valid path
        let result = get_binary_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_version_comparison() {
        // Test version comparison logic
        assert_eq!("0.4.9", CURRENT_VERSION);
    }
}
