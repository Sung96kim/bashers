use crate::utils::{colors::Colors, spinner};
use anyhow::{Context, Result};
use regex::Regex;
use std::process::Command;

const CRATES_IO_CRATE: &str = "bashers";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run() -> Result<()> {
    let mut colors = Colors::new();

    let latest_version = if spinner::should_show_spinner() {
        let mut sp = spinner::create_spinner("Checking for updates...");
        let result = get_latest_version();
        spinner::stop_spinner(sp.as_mut());
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

    colors.green()?;
    colors.print("Installing via cargo...")?;
    colors.reset()?;
    colors.println("")?;

    let status = Command::new("cargo")
        .args(["install", CRATES_IO_CRATE, "--force"])
        .status()
        .context("Failed to run cargo install")?;

    if !status.success() {
        anyhow::bail!("cargo install failed");
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
            &format!("https://crates.io/api/v1/crates/{}", CRATES_IO_CRATE),
        ])
        .output()
        .context("Failed to fetch latest version from crates.io")?;

    if !output.status.success() {
        anyhow::bail!("Failed to fetch latest version");
    }

    let response = String::from_utf8(output.stdout)?;
    let re =
        Regex::new(r#""newest_version"\s*:\s*"([^"]+)""#).context("Failed to create regex")?;

    let version = re
        .captures(&response)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .context("No newest_version in crates.io API response")?;

    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_from_crates_io_api() {
        let response = r#"{"crate":{"newest_version":"0.4.12"}}"#;
        let re = Regex::new(r#""newest_version"\s*:\s*"([^"]+)""#).unwrap();
        let version = re
            .captures(response)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());
        assert_eq!(version, Some("0.4.12".to_string()));
    }

    #[test]
    fn test_parse_version_with_whitespace() {
        let response = r#"{"newest_version" : "0.4.11"}"#;
        let re = Regex::new(r#""newest_version"\s*:\s*"([^"]+)""#).unwrap();
        let version = re
            .captures(response)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());
        assert_eq!(version, Some("0.4.11".to_string()));
    }

    #[test]
    fn test_parse_version_invalid_response() {
        let response = r#"{"crate":{"name":"bashers"}}"#;
        let re = Regex::new(r#""newest_version"\s*:\s*"([^"]+)""#).unwrap();
        let version = re
            .captures(response)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());
        assert_eq!(version, None);
    }

    #[test]
    fn test_version_constant() {
        assert_eq!(env!("CARGO_PKG_VERSION"), CURRENT_VERSION);
    }
}
