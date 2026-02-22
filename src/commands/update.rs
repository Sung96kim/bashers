use crate::utils::{colors, packages, project, spinner};
use anyhow::{Context, Result};
use std::process::Command;

pub fn run(package_patterns: &[String], dry_run: bool, auto_select: bool) -> Result<()> {
    let project_type = project::detect()?.context("No uv/poetry/cargo project found")?;

    if package_patterns.is_empty() {
        update_all(project_type, dry_run)?;
        return Ok(());
    }

    let all_packages = packages::list(project_type)?;
    let mut combined: Vec<String> = Vec::new();
    for pattern in package_patterns {
        let matches = packages::fuzzy_match(&all_packages, pattern)?;
        for m in matches {
            if !combined.contains(&m) {
                combined.push(m);
            }
        }
    }

    if combined.is_empty() {
        anyhow::bail!("No packages matched");
    }

    let selected: Vec<String> = if package_patterns.len() == 1 {
        let one = if dry_run || auto_select {
            packages::select_one_with_auto_select(combined, auto_select)?
        } else {
            packages::select_one(combined)?
        };
        vec![one]
    } else {
        let many = if dry_run || auto_select {
            packages::select_many_with_auto_select(combined, auto_select)?
        } else {
            packages::select_many(combined)?
        };
        if many.is_empty() {
            anyhow::bail!("No packages selected");
        }
        many
    };

    for pkg in &selected {
        colors::print_update(pkg);
    }
    update_packages(project_type, &selected, dry_run)?;

    Ok(())
}

fn update_packages(
    project_type: project::ProjectType,
    packages: &[String],
    dry_run: bool,
) -> Result<()> {
    if packages.is_empty() {
        return Ok(());
    }

    if dry_run {
        match project_type {
            project::ProjectType::Uv => {
                for p in packages {
                    println!("uv lock --upgrade-package \"{}\"", p);
                }
                println!("uv sync --all-extras");
            }
            project::ProjectType::Poetry => {
                let args: Vec<&str> = packages.iter().map(String::as_str).collect();
                println!("poetry update {}", args.join(" "));
            }
            project::ProjectType::Cargo => {
                let args: Vec<String> = packages
                    .iter()
                    .flat_map(|p| vec!["-p".to_string(), p.clone()])
                    .collect();
                println!("cargo update {}", args.join(" "));
            }
        }
        return Ok(());
    }

    match project_type {
        project::ProjectType::Uv => {
            let mut lock = Command::new("uv");
            lock.arg("lock");
            for p in packages {
                lock.args(["--upgrade-package", p]);
            }
            let status1 = spinner::run_with_spinner("Updating lockfile...", &mut lock)?;
            if !status1.success() {
                anyhow::bail!("uv lock failed");
            }
            let status2 = spinner::run_with_spinner(
                "Syncing...",
                Command::new("uv").args(["sync", "--all-extras"]),
            )?;
            if !status2.success() {
                anyhow::bail!("uv sync failed");
            }
            Ok(())
        }
        project::ProjectType::Poetry => {
            let status = spinner::run_with_spinner(
                "Updating packages...",
                Command::new("poetry").arg("update").args(packages),
            )?;
            if !status.success() {
                anyhow::bail!("poetry update failed");
            }
            Ok(())
        }
        project::ProjectType::Cargo => {
            let mut cmd = Command::new("cargo");
            cmd.arg("update");
            for p in packages {
                cmd.args(["-p", p]);
            }
            let status = spinner::run_with_spinner("Updating packages...", &mut cmd)?;
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
    fn test_update_packages_dry_run_uv() {
        let result = update_packages(ProjectType::Uv, &["test-package".into()], true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_dry_run_poetry() {
        let result = update_packages(ProjectType::Poetry, &["test-package".into()], true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_dry_run_cargo() {
        let result = update_packages(ProjectType::Cargo, &["test-package".into()], true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_dry_run_empty() {
        let result = update_packages(ProjectType::Cargo, &[], true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_dry_run_special_chars() {
        let result = update_packages(ProjectType::Cargo, &["test-package_v1.0".into()], true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_dry_run_multiple() {
        let result = update_packages(ProjectType::Cargo, &["pkg-a".into(), "pkg-b".into()], true);
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
    fn test_update_packages_output_format() {
        let result = update_packages(ProjectType::Cargo, &["test".into()], true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_output_format() {
        let result = update_all(ProjectType::Cargo, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_with_special_chars() {
        let result = update_packages(ProjectType::Cargo, &["test-package_v1.0".into()], true);
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
    fn test_update_packages_uv() {
        let result = update_packages(ProjectType::Uv, &["test-package".into()], true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_poetry() {
        let result = update_packages(ProjectType::Poetry, &["test-package".into()], true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_cargo() {
        let result = update_packages(ProjectType::Cargo, &["test-package".into()], true);
        assert!(result.is_ok());
    }
}
