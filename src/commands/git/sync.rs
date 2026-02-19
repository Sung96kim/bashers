use crate::utils::colors::Colors;
use crate::utils::spinner;
use anyhow::{Context, Result};
use spinoff::Color as SpinoffColor;
use std::io::{self, Write};
use std::process::{self, Command};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

const SEPARATOR: &str = "────────────────────────────────────────";

fn print_separator(colors: &mut Colors) -> io::Result<()> {
    colors.reset()?;
    colors.print(&format!("\n{}\n\n", SEPARATOR))?;
    colors.flush()
}

fn fail_cmd(cmd: &str) -> ! {
    let mut stderr = StandardStream::stderr(if atty::is(atty::Stream::Stderr) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });
    let _ = stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
    let _ = writeln!(&mut stderr, "✗ Command: `{}` failed.", cmd);
    let _ = stderr.reset();
    let _ = stderr.flush();
    process::exit(1);
}

pub fn run(current: bool, dry_run: bool) -> Result<()> {
    let branch = if current {
        get_current_branch()
            .context("Could not determine current branch. Are you in a git repository?")?
    } else {
        get_default_branch()
            .context("Could not determine default branch. Are you in a git repository?")?
    };

    let mut colors = Colors::new();

    if !current {
        if dry_run {
            println!("git checkout {}", branch);
        } else {
            let branch_clone = branch.clone();
            let spinner_msg = format!("Checking out [{}]", branch);
            let success_msg = format!("Checked out [{}]", branch);
            let output = spinner::run_with_completion(
                dry_run,
                &spinner_msg,
                &success_msg,
                Some(SpinoffColor::Red),
                || {
                    Command::new("git")
                        .args(["checkout", &branch_clone])
                        .output()
                },
                |o| o.status.success(),
            );
            match output {
                Ok(ref out) => {
                    if !out.status.success() {
                        spinner::print_failure_message(&format!("Checking out [{}]", branch));
                    }
                    print_pull_output(&mut colors, &out.stdout, &out.stderr)?;
                    if !out.status.success() {
                        fail_cmd(&format!("git checkout {}", branch));
                    }
                }
                Err(_) => fail_cmd(&format!("git checkout {}", branch)),
            }
        }
    }

    print_separator(&mut colors)?;

    let _did_stash = if dry_run {
        println!("git pull origin {}", branch);
        false
    } else {
        run_pull_step(&mut colors, &branch, dry_run)?
    };

    print_separator(&mut colors)?;

    if dry_run {
        println!("git fetch --all");
    } else {
        let output: std::result::Result<process::Output, io::Error> = spinner::run_with_completion(
            dry_run,
            "Fetching all",
            "Fetched all",
            Some(SpinoffColor::Green),
            || Command::new("git").args(["fetch", "--all"]).output(),
            |o| o.status.success(),
        );
        match output {
            Ok(ref out) => {
                if !out.status.success() {
                    spinner::print_failure_message("Fetching all");
                }
                print_pull_output(&mut colors, &out.stdout, &out.stderr)?;
                if !out.status.success() {
                    fail_cmd("git fetch --all");
                }
            }
            Err(_) => fail_cmd("git fetch --all"),
        }
    }

    spinner::print_success_message("Done.");

    Ok(())
}

fn run_pull_step(colors: &mut Colors, branch: &str, dry_run: bool) -> Result<bool> {
    let branch_clone = branch.to_string();
    let pull_spinner_msg = format!("Pulling origin [{}]", branch);
    let pull_success_msg = format!("Pulled origin [{}]", branch);
    let pull_cmd = format!("git pull origin {}", branch);

    let output = spinner::run_with_completion(
        dry_run,
        &pull_spinner_msg,
        &pull_success_msg,
        Some(SpinoffColor::Green),
        || {
            Command::new("git")
                .args(["pull", "origin", &branch_clone])
                .output()
        },
        |o| o.status.success(),
    );
    let output = match output {
        Ok(o) => o,
        Err(_) => fail_cmd(&pull_cmd),
    };
    if !output.status.success() {
        spinner::print_failure_message(&pull_spinner_msg);
    }
    print_pull_output(colors, &output.stdout, &output.stderr)?;
    if !output.status.success() {
        fail_cmd(&pull_cmd);
    }
    Ok(false)
}

fn is_fast_forward_summary_line(line: &str) -> bool {
    let line = line.trim();
    if line.is_empty() {
        return false;
    }
    if line.contains(" | ")
        && (line.contains('+') || line.contains('-'))
        && line.chars().any(|c| c.is_ascii_digit())
    {
        return true;
    }
    if line.starts_with(|c: char| c.is_ascii_digit()) && line.contains("files changed") {
        return true;
    }
    if line == "Fast-forward" {
        return true;
    }
    false
}

fn print_pull_output(colors: &mut Colors, stdout: &[u8], stderr: &[u8]) -> io::Result<()> {
    let has_output = !stdout.is_empty() || !stderr.is_empty();
    if has_output {
        colors.println("")?;
    }
    let print_lines = |colors: &mut Colors, data: &[u8]| -> io::Result<()> {
        let text = String::from_utf8_lossy(data);
        for line in text.lines() {
            let line = line.trim_end_matches('\r');
            if is_fast_forward_summary_line(line) {
                colors.yellow()?;
                colors.println(line)?;
                colors.reset()?;
            } else {
                colors.println(line)?;
            }
        }
        Ok(())
    };
    print_lines(colors, stdout)?;
    print_lines(colors, stderr)?;
    colors.flush()
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to run git branch --show-current")?;

    if output.status.success() {
        let branch = String::from_utf8(output.stdout)?.trim().to_string();
        if branch.is_empty() {
            anyhow::bail!("Not on a branch (detached HEAD)");
        }
        return Ok(branch);
    }
    anyhow::bail!("Could not determine current branch")
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
        if let Some(branch) = stdout.trim().strip_prefix("refs/remotes/origin/") {
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
    fn test_is_fast_forward_summary_line() {
        assert!(is_fast_forward_summary_line(" CHANGELOG.md | 6 ++++++"));
        assert!(is_fast_forward_summary_line(" Cargo.toml   | 2 +-"));
        assert!(is_fast_forward_summary_line(
            "2 files changed, 7 insertions(+), 1 deletion(-)"
        ));
        assert!(is_fast_forward_summary_line("Fast-forward"));
        assert!(!is_fast_forward_summary_line(
            "Your branch is behind 'origin/main' by 1 commit."
        ));
        assert!(!is_fast_forward_summary_line(""));
    }

    #[test]
    fn test_branch_parsing_edge_cases() {
        let cases = vec![
            ("refs/remotes/origin/main", Some("main")),
            ("refs/remotes/origin/master", Some("master")),
            ("refs/remotes/origin/feature-branch", Some("feature-branch")),
            ("invalid", None),
            ("refs/remotes/origin/", None),
        ];

        for (input, expected) in cases {
            let result = input.trim().strip_prefix("refs/remotes/origin/");

            match expected {
                Some(branch) => {
                    assert!(result.is_some());
                    assert_eq!(result.unwrap(), branch);
                }
                None => {
                    if input == "refs/remotes/origin/" {
                        assert_eq!(result, Some(""));
                    } else {
                        assert!(result.is_none());
                    }
                }
            }
        }
    }

    #[test]
    fn test_print_pull_output_empty_no_panic() {
        let mut colors = Colors::new();
        print_pull_output(&mut colors, b"", b"").unwrap();
    }

    #[test]
    fn test_print_pull_output_stdout_only_no_panic() {
        let mut colors = Colors::new();
        print_pull_output(&mut colors, b"Already on 'main'\n", b"").unwrap();
    }

    #[test]
    fn test_print_pull_output_stderr_only_no_panic() {
        let mut colors = Colors::new();
        print_pull_output(&mut colors, b"", b"From origin\n").unwrap();
    }

    #[test]
    fn test_print_pull_output_both_no_panic() {
        let mut colors = Colors::new();
        print_pull_output(
            &mut colors,
            b"Your branch is up to date.\n",
            b"From github.com:repo\n",
        )
        .unwrap();
    }

    #[test]
    fn test_print_pull_output_with_cr_no_panic() {
        let mut colors = Colors::new();
        print_pull_output(&mut colors, b"line with\r\n", b"").unwrap();
    }

    #[test]
    fn test_is_fast_forward_summary_line_more_cases() {
        assert!(is_fast_forward_summary_line(
            "1 files changed, 1 insertion(+)"
        ));
        assert!(is_fast_forward_summary_line(
            "10 files changed, 100 insertions(+), 50 deletions(-)"
        ));
        assert!(!is_fast_forward_summary_line("normal log line"));
        assert!(!is_fast_forward_summary_line(
            "Merge made by the 'recursive' strategy."
        ));
    }
}
