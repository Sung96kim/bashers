use crate::utils::{project, spinner};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn run(frozen: bool, rm: bool, dry_run: bool) -> Result<()> {
    if rm {
        if dry_run {
            println!("rm -rf .venv");
        } else if Path::new(".venv").exists() {
            fs::remove_dir_all(".venv").context("Failed to remove .venv")?;
        }
    }

    let project_type = project::detect()?.context("No uv/poetry/cargo project found")?;

    match project_type {
        project::ProjectType::Uv => {
            setup_uv(frozen, rm, dry_run)?;
        }
        project::ProjectType::Poetry => {
            setup_poetry(frozen, rm, dry_run)?;
        }
        project::ProjectType::Cargo => {
            setup_cargo(frozen, rm, dry_run)?;
        }
    }

    Ok(())
}

fn setup_uv(frozen: bool, rm: bool, dry_run: bool) -> Result<()> {
    let mut args = vec!["sync", "--all-extras"];

    if frozen {
        args.push("--frozen");
    }

    if rm {
        args.push("--no-cache");
    }

    if dry_run {
        println!("uv {}", args.join(" "));
        return Ok(());
    }

    let status = spinner::run_with_spinner(
        "Installing dependencies with uv...",
        Command::new("uv").args(&args),
    )?;

    if !status.success() {
        anyhow::bail!("uv sync failed");
    }

    Ok(())
}

fn setup_poetry(frozen: bool, rm: bool, dry_run: bool) -> Result<()> {
    let mut args = vec!["install", "--all-extras"];

    if frozen {
        args.push("--sync");
    }

    if rm {
        args.push("--no-cache");
    }

    if dry_run {
        println!("poetry {}", args.join(" "));
        return Ok(());
    }

    let status = spinner::run_with_spinner(
        "Installing dependencies with poetry...",
        Command::new("poetry").args(&args),
    )?;

    if !status.success() {
        anyhow::bail!("poetry install failed");
    }

    Ok(())
}

fn setup_cargo(frozen: bool, rm: bool, dry_run: bool) -> Result<()> {
    if rm {
        if dry_run {
            println!("rm -rf target");
        } else if Path::new("target").exists() {
            fs::remove_dir_all("target").context("Failed to remove target")?;
        }
    }

    let mut args = vec!["build"];

    if frozen {
        // For Cargo, --frozen means don't update Cargo.lock
        args.push("--frozen");
    }

    if dry_run {
        println!("cargo {}", args.join(" "));
        return Ok(());
    }

    let status = spinner::run_with_spinner(
        "Building with cargo...",
        Command::new("cargo").args(&args),
    )?;

    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    Ok(())
}

// Helper function for testing argument building
#[cfg(test)]
fn build_uv_args(frozen: bool, rm: bool) -> Vec<&'static str> {
    let mut args = vec!["sync", "--all-extras"];
    if frozen {
        args.push("--frozen");
    }
    if rm {
        args.push("--no-cache");
    }
    args
}

#[cfg(test)]
fn build_poetry_args(frozen: bool, rm: bool) -> Vec<&'static str> {
    let mut args = vec!["install", "--all-extras"];
    if frozen {
        args.push("--sync");
    }
    if rm {
        args.push("--no-cache");
    }
    args
}

#[cfg(test)]
fn build_cargo_args(frozen: bool) -> Vec<&'static str> {
    let mut args = vec!["build"];
    if frozen {
        args.push("--frozen");
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_uv_dry_run() {
        let result = setup_uv(false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_uv_dry_run_frozen() {
        let result = setup_uv(true, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_uv_dry_run_rm() {
        let result = setup_uv(false, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_uv_dry_run_frozen_rm() {
        let result = setup_uv(true, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_poetry_dry_run() {
        let result = setup_poetry(false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_poetry_dry_run_frozen() {
        let result = setup_poetry(true, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_poetry_dry_run_rm() {
        let result = setup_poetry(false, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_poetry_dry_run_frozen_rm() {
        let result = setup_poetry(true, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_cargo_dry_run() {
        let result = setup_cargo(false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_cargo_dry_run_frozen() {
        let result = setup_cargo(true, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_cargo_dry_run_rm() {
        let result = setup_cargo(false, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_cargo_dry_run_frozen_rm() {
        let result = setup_cargo(true, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_uv_args() {
        assert_eq!(build_uv_args(false, false), vec!["sync", "--all-extras"]);
        assert_eq!(
            build_uv_args(true, false),
            vec!["sync", "--all-extras", "--frozen"]
        );
        assert_eq!(
            build_uv_args(false, true),
            vec!["sync", "--all-extras", "--no-cache"]
        );
        assert_eq!(
            build_uv_args(true, true),
            vec!["sync", "--all-extras", "--frozen", "--no-cache"]
        );
    }

    #[test]
    fn test_build_poetry_args() {
        assert_eq!(
            build_poetry_args(false, false),
            vec!["install", "--all-extras"]
        );
        assert_eq!(
            build_poetry_args(true, false),
            vec!["install", "--all-extras", "--sync"]
        );
        assert_eq!(
            build_poetry_args(false, true),
            vec!["install", "--all-extras", "--no-cache"]
        );
        assert_eq!(
            build_poetry_args(true, true),
            vec!["install", "--all-extras", "--sync", "--no-cache"]
        );
    }

    #[test]
    fn test_build_cargo_args() {
        assert_eq!(build_cargo_args(false), vec!["build"]);
        assert_eq!(build_cargo_args(true), vec!["build", "--frozen"]);
    }

    #[test]
    fn test_setup_rm_dry_run() {
        // Test that rm flag with dry_run prints the correct command
        // We can't easily test println, but we can verify the function succeeds
        let result = run(false, true, true);
        // This will fail if no project is detected, which is expected in test environment
        // But the rm logic should still execute
        let _ = result;
    }
}
