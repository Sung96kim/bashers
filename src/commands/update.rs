use crate::utils::{colors, multi_progress, packages, project, spinner};
use anyhow::{Context, Result};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;
use std::process::Command;

fn fmt_version(v: &str) -> String {
    if v.starts_with('v') {
        v.to_string()
    } else {
        format!("v{}", v)
    }
}

fn version_change(before: &str, after: &str) -> colors::VersionChange {
    match cmp_version(before, after) {
        Ordering::Less => colors::VersionChange::Upgraded,
        Ordering::Equal => colors::VersionChange::Unchanged,
        Ordering::Greater => colors::VersionChange::Downgraded,
    }
}

fn cmp_version(a: &str, b: &str) -> Ordering {
    let a = a.trim_start_matches('v');
    let b = b.trim_start_matches('v');
    let parts_a: Vec<u64> = a
        .split('.')
        .filter_map(|s| s.split('-').next())
        .filter_map(|s| s.parse().ok())
        .collect();
    let parts_b: Vec<u64> = b
        .split('.')
        .filter_map(|s| s.split('-').next())
        .filter_map(|s| s.parse().ok())
        .collect();
    for (pa, pb) in parts_a.iter().zip(parts_b.iter()) {
        match pa.cmp(pb) {
            Ordering::Equal => continue,
            o => return o,
        }
    }
    parts_a.len().cmp(&parts_b.len())
}

pub fn run(
    package_patterns: &[String],
    dry_run: bool,
    auto_select: bool,
    verbose: bool,
) -> Result<()> {
    let project_type = project::detect()?.context("No uv/poetry/cargo project found")?;

    if package_patterns.is_empty() {
        update_all(project_type, dry_run, verbose)?;
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

    update_packages(project_type, &selected, dry_run, verbose)?;

    Ok(())
}

fn update_packages(
    project_type: project::ProjectType,
    packages: &[String],
    dry_run: bool,
    verbose: bool,
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

    let stdout_buf = RefCell::new(Vec::<u8>::new());
    let stderr_buf = RefCell::new(Vec::<u8>::new());

    let run_update = || -> Result<()> {
        let forward = |out: &std::process::Output| {
            if verbose {
                stdout_buf.borrow_mut().extend_from_slice(&out.stdout);
                stderr_buf.borrow_mut().extend_from_slice(&out.stderr);
            }
        };
        match project_type {
            project::ProjectType::Uv => {
                let mut lock = Command::new("uv");
                lock.arg("lock");
                for p in packages {
                    lock.args(["--upgrade-package", p]);
                }
                let out1 = lock.output().context("Failed to run uv lock")?;
                forward(&out1);
                if !out1.status.success() {
                    anyhow::bail!("uv lock failed");
                }
                let out2 = Command::new("uv")
                    .args(["sync", "--all-extras"])
                    .output()
                    .context("Failed to run uv sync")?;
                forward(&out2);
                if !out2.status.success() {
                    anyhow::bail!("uv sync failed");
                }
                Ok(())
            }
            project::ProjectType::Poetry => {
                let out = Command::new("poetry")
                    .arg("update")
                    .args(packages)
                    .output()
                    .context("Failed to run poetry update")?;
                forward(&out);
                if !out.status.success() {
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
                let out = cmd.output().context("Failed to run cargo update")?;
                forward(&out);
                if !out.status.success() {
                    anyhow::bail!("cargo update failed");
                }
                Ok(())
            }
        }
    };

    let result = if spinner::should_show_spinner() {
        let before_versions: HashMap<String, Option<String>> = packages
            .iter()
            .map(|p| {
                (
                    p.clone(),
                    packages::get_installed_version(project_type, p)
                        .ok()
                        .flatten(),
                )
            })
            .collect();
        let multi = multi_progress::multi_progress_stderr();
        multi_progress::run_spinners_then_single_op(
            &multi,
            packages,
            |one_indexed, total, pkg| {
                if atty::is(atty::Stream::Stderr) {
                    format!(
                        "[{}/{}] {}[{}]{} ",
                        one_indexed,
                        total,
                        colors::ANSI_GREEN,
                        pkg,
                        colors::ANSI_RESET
                    )
                } else {
                    format!("[{}/{}] [{}] ", one_indexed, total, pkg)
                }
            },
            run_update,
            |pkg, success| {
                if success {
                    let before = before_versions
                        .get(pkg)
                        .and_then(|v| v.as_deref())
                        .map(fmt_version)
                        .unwrap_or_else(|| "?".to_string());
                    let after = packages::get_installed_version(project_type, pkg)
                        .ok()
                        .flatten()
                        .map(|v| fmt_version(&v))
                        .unwrap_or_else(|| "?".to_string());
                    let change = version_change(&before, &after);
                    colors::format_bumped_message_colored(&before, &after, change)
                } else {
                    let failed = "Failed";
                    if atty::is(atty::Stream::Stderr) {
                        format!("{}{}{}", colors::ANSI_RED, failed, colors::ANSI_RESET)
                    } else {
                        failed.to_string()
                    }
                }
            },
        )
    } else {
        run_update()
    };

    if verbose {
        let _ = std::io::stdout().write_all(&stdout_buf.borrow());
        let _ = std::io::stderr().write_all(&stderr_buf.borrow());
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
    }

    result
}

fn update_all(project_type: project::ProjectType, dry_run: bool, verbose: bool) -> Result<()> {
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

    let stdout_buf = RefCell::new(Vec::<u8>::new());
    let stderr_buf = RefCell::new(Vec::<u8>::new());

    let forward = |out: &std::process::Output| {
        if verbose {
            stdout_buf.borrow_mut().extend_from_slice(&out.stdout);
            stderr_buf.borrow_mut().extend_from_slice(&out.stderr);
        }
    };

    let run_all = || -> Result<()> {
        match project_type {
            project::ProjectType::Uv => {
                let out1 = Command::new("uv")
                    .args(["lock", "--upgrade"])
                    .output()
                    .context("Failed to run uv lock")?;
                forward(&out1);
                if !out1.status.success() {
                    anyhow::bail!("uv lock failed");
                }
                let out2 = Command::new("uv")
                    .args(["sync", "--all-extras"])
                    .output()
                    .context("Failed to run uv sync")?;
                forward(&out2);
                if !out2.status.success() {
                    anyhow::bail!("uv sync failed");
                }
                Ok(())
            }
            project::ProjectType::Poetry => {
                let out = Command::new("poetry")
                    .arg("update")
                    .output()
                    .context("Failed to run poetry update")?;
                forward(&out);
                if !out.status.success() {
                    anyhow::bail!("poetry update failed");
                }
                Ok(())
            }
            project::ProjectType::Cargo => {
                let out = Command::new("cargo")
                    .arg("update")
                    .output()
                    .context("Failed to run cargo update")?;
                forward(&out);
                if !out.status.success() {
                    anyhow::bail!("cargo update failed");
                }
                Ok(())
            }
        }
    };

    let result = if spinner::should_show_spinner() {
        let multi = multi_progress::multi_progress_stderr();
        multi_progress::run_header_spinner(
            &multi,
            "Updating all packages...",
            "✓ Updated",
            "✗ Failed",
            run_all,
        )
    } else {
        run_all()
    };

    if verbose {
        let _ = std::io::stdout().write_all(&stdout_buf.borrow());
        let _ = std::io::stderr().write_all(&stderr_buf.borrow());
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::project::ProjectType;

    #[test]
    fn test_update_packages_dry_run_uv() {
        let result = update_packages(ProjectType::Uv, &["test-package".into()], true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_dry_run_poetry() {
        let result = update_packages(ProjectType::Poetry, &["test-package".into()], true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_dry_run_cargo() {
        let result = update_packages(ProjectType::Cargo, &["test-package".into()], true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_dry_run_empty() {
        let result = update_packages(ProjectType::Cargo, &[], true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_dry_run_special_chars() {
        let result = update_packages(
            ProjectType::Cargo,
            &["test-package_v1.0".into()],
            true,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_dry_run_multiple() {
        let result = update_packages(
            ProjectType::Cargo,
            &["pkg-a".into(), "pkg-b".into()],
            true,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_dry_run_uv() {
        let result = update_all(ProjectType::Uv, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_dry_run_poetry() {
        let result = update_all(ProjectType::Poetry, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_dry_run_cargo() {
        let result = update_all(ProjectType::Cargo, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_output_format() {
        let result = update_packages(ProjectType::Cargo, &["test".into()], true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_output_format() {
        let result = update_all(ProjectType::Cargo, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_with_special_chars() {
        let result = update_packages(
            ProjectType::Cargo,
            &["test-package_v1.0".into()],
            true,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_uv() {
        let result = update_all(ProjectType::Uv, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_all_poetry() {
        let result = update_all(ProjectType::Poetry, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_uv() {
        let result = update_packages(ProjectType::Uv, &["test-package".into()], true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_poetry() {
        let result = update_packages(ProjectType::Poetry, &["test-package".into()], true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_packages_cargo() {
        let result = update_packages(ProjectType::Cargo, &["test-package".into()], true, false);
        assert!(result.is_ok());
    }
}
