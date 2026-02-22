use crate::utils::{colors::Colors, spinner};
use anyhow::{Context, Result};
use regex::Regex;
use std::env;
use std::path::Path;
use std::process::Command;

const CRATES_IO_CRATE: &str = "bashers";
const PYPI_PACKAGE: &str = "bashers";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallKind {
    Cargo,
    Pip,
}

pub fn detect_install_kind() -> InstallKind {
    if let Ok(exe) = env::current_exe() {
        let path_str = exe.to_string_lossy();
        if path_str.contains(".cargo") {
            return InstallKind::Cargo;
        }
        if path_str.contains(".local") && (path_str.contains("bin") || path_str.contains("Scripts"))
        {
            return InstallKind::Pip;
        }
        if let Ok(venv) = env::var("VIRTUAL_ENV") {
            let venv_path = Path::new(&venv);
            if exe.strip_prefix(venv_path).is_ok() {
                return InstallKind::Pip;
            }
        }
    }
    if cargo_has_bashers() && !pip_has_bashers() {
        return InstallKind::Cargo;
    }
    if pip_has_bashers() {
        return InstallKind::Pip;
    }
    InstallKind::Cargo
}

fn cargo_has_bashers() -> bool {
    Command::new("cargo")
        .args(["install", "--list"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().any(|l| l.trim().starts_with("bashers ")))
        .unwrap_or(false)
}

fn pip_has_bashers() -> bool {
    let mut cmd = Command::new("pip");
    cmd.args(["show", PYPI_PACKAGE]);
    cmd.output()
        .or_else(|_| {
            let mut uv = Command::new("uv");
            uv.args(["pip", "show", PYPI_PACKAGE]);
            uv.output()
        })
        .ok()
        .filter(|o| o.status.success())
        .is_some()
}

pub fn run() -> Result<()> {
    let kind = detect_install_kind();
    let mut colors = Colors::new();

    let latest_version = if spinner::should_show_spinner() {
        let mut sp = spinner::create_spinner("Checking for updates...");
        let result = match kind {
            InstallKind::Cargo => get_latest_version_crates_io(),
            InstallKind::Pip => get_latest_version_pypi(),
        };
        spinner::stop_spinner(sp.as_mut());
        result?
    } else {
        colors.green()?;
        colors.print("Checking for updates...")?;
        colors.reset()?;
        colors.println("")?;
        match kind {
            InstallKind::Cargo => get_latest_version_crates_io()?,
            InstallKind::Pip => get_latest_version_pypi()?,
        }
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

    let (install_msg, success) = match kind {
        InstallKind::Cargo => {
            colors.green()?;
            colors.print("Installing via cargo...")?;
            colors.reset()?;
            colors.println("")?;
            let status = Command::new("cargo")
                .args(["install", CRATES_IO_CRATE, "--force"])
                .status()
                .context("Failed to run cargo install")?;
            ("cargo install", status.success())
        }
        InstallKind::Pip => {
            colors.green()?;
            colors.print("Installing via pip...")?;
            colors.reset()?;
            colors.println("")?;
            let status = run_pip_upgrade();
            ("pip install --upgrade", status)
        }
    };

    if !success {
        anyhow::bail!("{} failed", install_msg);
    }

    colors.green()?;
    colors.println(&format!("Successfully updated to v{}", latest_version))?;
    colors.reset()?;

    Ok(())
}

fn run_pip_upgrade() -> bool {
    let pip = Command::new("pip")
        .args(["install", "--upgrade", PYPI_PACKAGE])
        .status();
    if let Ok(s) = pip {
        return s.success();
    }
    Command::new("uv")
        .args(["pip", "install", "--upgrade", PYPI_PACKAGE])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn get_latest_version_crates_io() -> Result<String> {
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
    let re = Regex::new(r#""newest_version"\s*:\s*"([^"]+)""#).context("Failed to create regex")?;

    let version = re
        .captures(&response)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .context("No newest_version in crates.io API response")?;

    Ok(version)
}

fn get_latest_version_pypi() -> Result<String> {
    let output = Command::new("curl")
        .args([
            "-s",
            &format!("https://pypi.org/pypi/{}/json", PYPI_PACKAGE),
        ])
        .output()
        .context("Failed to fetch latest version from PyPI")?;

    if !output.status.success() {
        anyhow::bail!("Failed to fetch latest version from PyPI");
    }

    let response = String::from_utf8(output.stdout)?;
    let re = Regex::new(r#""version"\s*:\s*"([^"]+)""#).context("Failed to create regex")?;
    let version = re
        .captures(&response)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .context("No version in PyPI API response (package may not exist)")?;
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

    #[test]
    fn test_parse_version_pypi_response() {
        let response = r#"{"info":{"version":"0.8.5","name":"bashers"}}"#;
        let re = Regex::new(r#""version"\s*:\s*"([^"]+)""#).unwrap();
        let version = re
            .captures(response)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());
        assert_eq!(version, Some("0.8.5".to_string()));
    }
}
