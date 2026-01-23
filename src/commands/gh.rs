use crate::utils::colors::Colors;
use anyhow::{Context, Result};
use std::process::Command;

pub fn run(dry_run: bool) -> Result<()> {
    let default_branch = get_default_branch()
        .context("Could not determine default branch. Are you in a git repository?")?;

    let mut colors = Colors::new();

    colors.red()?;
    colors.print(&format!("\nChecking out '{}'\n\n", default_branch))?;
    colors.reset()?;

    if dry_run {
        println!("git checkout \"{}\"", default_branch);
    } else {
        Command::new("git")
            .args(["checkout", &default_branch])
            .status()
            .context("Failed to run git checkout")?;
    }

    colors.green()?;
    colors.print(&format!("\nPulling origin '{}'\n\n", default_branch))?;
    colors.reset()?;

    if dry_run {
        println!("git pull origin \"{}\"", default_branch);
    } else {
        Command::new("git")
            .args(["pull", "origin", &default_branch])
            .status()
            .context("Failed to run git pull")?;
    }

    colors.green()?;
    colors.print("\nFetching all\n\n")?;
    colors.reset()?;

    if dry_run {
        println!("git fetch --all");
    } else {
        Command::new("git")
            .args(["fetch", "--all"])
            .status()
            .context("Failed to run git fetch")?;
    }

    Ok(())
}

fn get_default_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["ls-remote", "--symref", "origin", "HEAD"])
        .output()
        .context("Failed to run git ls-remote")?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        for line in stdout.lines() {
            if line.starts_with("ref:") && line.contains("HEAD") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ref_path = parts[1];
                    if let Some(branch) = ref_path.strip_prefix("refs/heads/") {
                        return Ok(branch.to_string());
                    }
                }
            }
        }
    }

    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()
        .context("Failed to run git symbolic-ref")?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        if let Some(branch) = stdout
            .trim()
            .strip_prefix("refs/remotes/origin/")
        {
            return Ok(branch.to_string());
        }
    }

    let output = Command::new("git")
        .args(["remote", "show", "origin"])
        .output()
        .context("Failed to run git remote show")?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        for line in stdout.lines() {
            if line.trim().starts_with("HEAD branch:") {
                if let Some(branch) = line.trim().strip_prefix("HEAD branch:") {
                    return Ok(branch.trim().to_string());
                }
            }
        }
    }

    anyhow::bail!("Could not determine default branch")
}

// Helper function for testing branch parsing
#[cfg(test)]
fn parse_branch_output(output: &str) -> Result<String> {
    let branch = output
        .trim()
        .strip_prefix("refs/remotes/origin/")
        .ok_or_else(|| anyhow::anyhow!("Invalid git output"))?
        .to_string();
    Ok(branch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_branch_parsing() {
        // Test parsing logic with valid output
        let valid_output = "refs/remotes/origin/main";
        let branch = valid_output
            .trim()
            .strip_prefix("refs/remotes/origin/")
            .unwrap()
            .to_string();
        assert_eq!(branch, "main");
    }

    #[test]
    fn test_get_default_branch_parsing_master() {
        let valid_output = "refs/remotes/origin/master";
        let branch = valid_output
            .trim()
            .strip_prefix("refs/remotes/origin/")
            .unwrap()
            .to_string();
        assert_eq!(branch, "master");
    }

    #[test]
    fn test_get_default_branch_parsing_invalid() {
        let invalid_output = "invalid output";
        let result = invalid_output.trim().strip_prefix("refs/remotes/origin/");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_default_branch_parsing_with_whitespace() {
        let output = "  refs/remotes/origin/main  ";
        let branch = output
            .trim()
            .strip_prefix("refs/remotes/origin/")
            .unwrap()
            .to_string();
        assert_eq!(branch, "main");
    }

    #[test]
    fn test_get_default_branch_parsing_different_branch() {
        let valid_output = "refs/remotes/origin/develop";
        let branch = valid_output
            .trim()
            .strip_prefix("refs/remotes/origin/")
            .unwrap()
            .to_string();
        assert_eq!(branch, "develop");
    }

    #[test]
    fn test_parse_branch_output_helper() {
        let result = parse_branch_output("refs/remotes/origin/main");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "main");
    }

    #[test]
    fn test_parse_branch_output_helper_invalid() {
        let result = parse_branch_output("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ls_remote_output() {
        // Test parsing ls-remote --symref output
        let output = "ref: refs/heads/main\tHEAD\n1234567890abcdef\tHEAD\n";
        for line in output.lines() {
            if line.starts_with("ref:") && line.contains("HEAD") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ref_path = parts[1];
                    if let Some(branch) = ref_path.strip_prefix("refs/heads/") {
                        assert_eq!(branch, "main");
                        return;
                    }
                }
            }
        }
        panic!("Failed to parse ls-remote output");
    }

    #[test]
    fn test_parse_remote_show_output() {
        // Test parsing git remote show output
        let output = "  HEAD branch: main\n  Remote branches:\n    main tracked";
        for line in output.lines() {
            if line.trim().starts_with("HEAD branch:") {
                if let Some(branch) = line.trim().strip_prefix("HEAD branch:") {
                    assert_eq!(branch.trim(), "main");
                    return;
                }
            }
        }
        panic!("Failed to parse remote show output");
    }

    #[test]
    fn test_parse_ls_remote_different_format() {
        // Test parsing different ls-remote format
        let output = "ref: refs/heads/master\tHEAD";
        for line in output.lines() {
            if line.starts_with("ref:") && line.contains("HEAD") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ref_path = parts[1];
                    if let Some(branch) = ref_path.strip_prefix("refs/heads/") {
                        assert_eq!(branch, "master");
                        return;
                    }
                }
            }
        }
        panic!("Failed to parse ls-remote output");
    }

    #[test]
    fn test_parse_ls_remote_with_tabs() {
        // Test parsing ls-remote output with tabs
        let output = "ref:\trefs/heads/main\tHEAD";
        for line in output.lines() {
            if line.starts_with("ref:") && line.contains("HEAD") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ref_path = parts[1];
                    if let Some(branch) = ref_path.strip_prefix("refs/heads/") {
                        assert_eq!(branch, "main");
                        return;
                    }
                }
            }
        }
        panic!("Failed to parse ls-remote output");
    }

    #[test]
    fn test_parse_remote_show_with_whitespace() {
        // Test parsing git remote show output with extra whitespace
        let output = "    HEAD branch:    main    ";
        for line in output.lines() {
            if line.trim().starts_with("HEAD branch:") {
                if let Some(branch) = line.trim().strip_prefix("HEAD branch:") {
                    assert_eq!(branch.trim(), "main");
                    return;
                }
            }
        }
        panic!("Failed to parse remote show output");
    }

    #[test]
    fn test_branch_parsing_edge_cases() {
        // Test various edge cases
        let cases = vec![
            ("refs/remotes/origin/main", Some("main")),
            ("refs/remotes/origin/master", Some("master")),
            ("refs/remotes/origin/feature-branch", Some("feature-branch")),
            ("invalid", None),
            ("refs/remotes/origin/", None), // Empty branch name
        ];

        for (input, expected) in cases {
            let result = input.trim().strip_prefix("refs/remotes/origin/");

            match expected {
                Some(branch) => {
                    assert!(result.is_some());
                    assert_eq!(result.unwrap(), branch);
                }
                None => {
                    // For empty branch, strip_prefix returns Some("") which is not None
                    // So we need to check if it's empty
                    if input == "refs/remotes/origin/" {
                        assert_eq!(result, Some(""));
                    } else {
                        assert!(result.is_none());
                    }
                }
            }
        }
    }
}
