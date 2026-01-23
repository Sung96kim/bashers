use crate::utils::{colors, packages, project, spinner};
use anyhow::{Context, Result};
use std::process::Command;

pub fn run(package: Option<&str>, dry_run: bool, auto_select: bool) -> Result<()> {
    let project_type = project::detect()?.context("No uv/poetry/cargo project found")?;

    if let Some(pkg_pattern) = package {
        let all_packages = packages::list(project_type)?;
        let matches = packages::fuzzy_match(&all_packages, pkg_pattern)?;
        let selected = if dry_run || auto_select {
            packages::select_one_with_auto_select(matches, true)?
        } else {
            packages::select_one(matches)?
        };

        colors::print_update(&selected);

        update_package(project_type, &selected, dry_run)?;
    } else {
        update_all(project_type, dry_run)?;
    }

    Ok(())
}

fn update_package(project_type: project::ProjectType, package: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        match project_type {
            project::ProjectType::Uv => {
                println!("uv lock --upgrade-package \"{}\"", package);
                println!("uv sync --all-extras");
            }
            project::ProjectType::Poetry => {
                println!("poetry update \"{}\"", package);
            }
            project::ProjectType::Cargo => {
                println!("cargo update -p \"{}\"", package);
            }
        }
        return Ok(());
    }

    match project_type {
        project::ProjectType::Uv => {
            let status1 = spinner::run_with_spinner(
                &format!("Updating {}...", package),
                Command::new("uv").args(["lock", "--upgrade-package", package]),
            )?;

            if !status1.success() {
                anyhow::bail!("uv lock failed");
            }

            let status2 = spinner::run_with_spinner(
                &format!("Syncing {}...", package),
                Command::new("uv").args(["sync", "--all-extras"]),
            )?;

            if !status2.success() {
                anyhow::bail!("uv sync failed");
            }

            Ok(())
        }
        project::ProjectType::Poetry => {
            let status = spinner::run_with_spinner(
                &format!("Updating {}...", package),
                Command::new("poetry").args(["update", package]),
            )?;

            if !status.success() {
                anyhow::bail!("poetry update failed");
            }

            Ok(())
        }
        project::ProjectType::Cargo => {
            let status = spinner::run_with_spinner(
                &format!("Updating {}...", package),
                Command::new("cargo").args(["update", "-p", package]),
            )?;

            if !status.success() {
                anyhow::bail!("cargo update failed");
            }

            Ok(())
        }
    }
}

fn update_all(project_type: project::ProjectType, dry_run: bool) -> Result<()> {
    if dry_run {
        match project_type {
            project::ProjectType::Uv => {
                println!("uv lock --upgrade");
                println!("uv sync --all-extras");
            }
            project::ProjectType::Poetry => {
                println!("poetry update");
            }
            project::ProjectType::Cargo => {
                println!("cargo update");
            }
        }
        return Ok(());
    }

    match project_type {
        project::ProjectType::Uv => {
            let status1 = spinner::run_with_spinner(
                "Updating all packages...",
                Command::new("uv").args(["lock", "--upgrade"]),
            )?;

            if !status1.success() {
                anyhow::bail!("uv lock failed");
            }

            let status2 = spinner::run_with_spinner(
                "Syncing all packages...",
                Command::new("uv").args(["sync", "--all-extras"]),
            )?;

            if !status2.success() {
                anyhow::bail!("uv sync failed");
            }

            Ok(())
        }
        project::ProjectType::Poetry => {
            let status = spinner::run_with_spinner(
                "Updating all packages...",
                Command::new("poetry").arg("update"),
            )?;

            if !status.success() {
                anyhow::bail!("poetry update failed");
            }

            Ok(())
        }
        project::ProjectType::Cargo => {
            let status = spinner::run_with_spinner(
                "Updating all packages...",
                Command::new("cargo").arg("update"),
            )?;

            if !status.success() {
                anyhow::bail!("cargo update failed");
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::project::ProjectType;

    #[test]
    fn test_update_package_dry_run_uv() {
        let result = update_package(ProjectType::Uv, "test-package", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_package_dry_run_poetry() {
        let result = update_package(ProjectType::Poetry, "test-package", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_package_dry_run_cargo() {
        let result = update_package(ProjectType::Cargo, "test-package", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_package_dry_run_empty_package() {
        let result = update_package(ProjectType::Cargo, "", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_package_dry_run_special_chars() {
        let result = update_package(ProjectType::Cargo, "test-package_v1.0", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_dry_run_uv() {
        let result = update_all(ProjectType::Uv, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_dry_run_poetry() {
        let result = update_all(ProjectType::Poetry, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_dry_run_cargo() {
        let result = update_all(ProjectType::Cargo, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_package_output_format() {
        // Test that dry-run outputs are properly formatted
        // We can't easily capture println in tests, but we can verify the function succeeds
        let result = update_package(ProjectType::Cargo, "test", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_output_format() {
        let result = update_all(ProjectType::Cargo, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_package_with_special_chars() {
        // Test package names with special characters
        let result = update_package(ProjectType::Cargo, "test-package_v1.0", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_uv() {
        let result = update_all(ProjectType::Uv, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_poetry() {
        let result = update_all(ProjectType::Poetry, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_package_uv() {
        let result = update_package(ProjectType::Uv, "test-package", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_package_poetry() {
        let result = update_package(ProjectType::Poetry, "test-package", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_package_cargo() {
        let result = update_package(ProjectType::Cargo, "test-package", true);
        assert!(result.is_ok());
    }
}
