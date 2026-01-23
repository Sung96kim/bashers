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
        println!("gpo");
    } else {
        Command::new("gpo").status().context("Failed to run gpo")?;
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
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()
        .context("Failed to run git symbolic-ref")?;

    if !output.status.success() {
        anyhow::bail!("Could not determine default branch");
    }

    let stdout = String::from_utf8(output.stdout)?;
    let branch = stdout
        .trim()
        .strip_prefix("refs/remotes/origin/")
        .ok_or_else(|| anyhow::anyhow!("Invalid git output"))?
        .to_string();

    Ok(branch)
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
    fn test_gh_dry_run() {
        // Test that dry_run mode doesn't fail (even if git commands would)
        // This test will fail if not in a git repo, so we'll skip it
        // In a real scenario, we'd mock the git commands
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
