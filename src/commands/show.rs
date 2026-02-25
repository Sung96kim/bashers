use crate::utils::project;
use anyhow::{Context, Result};
use regex::Regex;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "gui", derive(serde::Serialize, serde::Deserialize))]
pub struct DependencyInfo {
    pub name: String,
    pub version: Option<String>,
}

pub fn parse_dependency_lines(lines: &[String]) -> Vec<DependencyInfo> {
    lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let cleaned = line
                .replace("├── ", "")
                .replace("└── ", "")
                .replace("│   ", "")
                .replace("│", "");
            let cleaned = cleaned.trim().to_string();
            if cleaned.is_empty() {
                return None;
            }
            let parts: Vec<&str> = cleaned.split_whitespace().collect();
            let name = parts.first().unwrap_or(&"").to_string();
            if name.is_empty() {
                return None;
            }
            Some(DependencyInfo {
                name,
                version: parts.get(1).map(|s| s.to_string()),
            })
        })
        .collect()
}

pub fn get_dependency_output(patterns: &[String]) -> Result<(project::ProjectType, Vec<String>)> {
    let project_type = project::detect()?.context("No uv/poetry/cargo project found")?;
    let (program, args): (&str, &[&str]) = match project_type {
        project::ProjectType::Uv => ("uv", &["pip", "list"]),
        project::ProjectType::Poetry => ("poetry", &["show"]),
        project::ProjectType::Cargo => ("cargo", &["tree", "--depth", "1"]),
    };

    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .output()
        .with_context(|| format!("Failed to run {} {}", program, args.join(" ")))?;

    if !output.status.success() {
        anyhow::bail!("{} {} failed", program, args.join(" "));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<String> = stdout.lines().map(String::from).collect();

    let filtered = if patterns.is_empty() {
        lines
    } else {
        lines
            .into_iter()
            .filter(|line| matches_any_pattern(line, patterns))
            .collect()
    };

    Ok((project_type, filtered))
}

pub fn run(patterns: &[String]) -> Result<()> {
    let project_type = project::detect()?.context("No uv/poetry/cargo project found")?;

    match project_type {
        project::ProjectType::Uv => show_filtered("uv", &["pip", "list"], patterns),
        project::ProjectType::Poetry => show_filtered("poetry", &["show"], patterns),
        project::ProjectType::Cargo => show_filtered("cargo", &["tree"], patterns),
    }
}

fn show_filtered(program: &str, args: &[&str], patterns: &[String]) -> Result<()> {
    let mut cmd = Command::new(program);
    cmd.args(args);

    if patterns.is_empty() {
        let status = cmd
            .status()
            .with_context(|| format!("Failed to run {} {}", program, args.join(" ")))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    let output = cmd
        .stdout(Stdio::piped())
        .output()
        .with_context(|| format!("Failed to run {} {}", program, args.join(" ")))?;

    if !output.status.success() {
        anyhow::bail!("{} {} failed", program, args.join(" "));
    }

    let stdout = String::from_utf8(output.stdout)?;

    for line in stdout.lines() {
        if matches_any_pattern(line, patterns) {
            println!("{}", line);
        }
    }

    Ok(())
}

fn matches_any_pattern(text: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .any(|pattern| regex_match_case_insensitive(text, pattern))
}

fn regex_match_case_insensitive(text: &str, pattern: &str) -> bool {
    let escaped = regex::escape(pattern);
    match Regex::new(&format!("(?i){}", escaped)) {
        Ok(re) => re.is_match(text),
        Err(_) => text.to_lowercase().contains(&pattern.to_lowercase()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_match_case_insensitive_exact() {
        assert!(regex_match_case_insensitive("clap", "clap"));
        assert!(regex_match_case_insensitive("CLAP", "clap"));
        assert!(regex_match_case_insensitive("clap", "CLAP"));
    }

    #[test]
    fn test_regex_match_case_insensitive_partial() {
        assert!(regex_match_case_insensitive("clap-derive", "clap"));
        assert!(regex_match_case_insensitive("anyhow", "any"));
        assert!(regex_match_case_insensitive("regex", "reg"));
    }

    #[test]
    fn test_regex_match_case_insensitive_no_match() {
        assert!(!regex_match_case_insensitive("clap", "nonexistent"));
        assert!(!regex_match_case_insensitive("anyhow", "clap"));
    }

    #[test]
    fn test_regex_match_case_insensitive_special_chars() {
        assert!(regex_match_case_insensitive("test.package", "test.package"));
        assert!(regex_match_case_insensitive("test+package", "test+package"));
        assert!(regex_match_case_insensitive("test*package", "test*package"));
    }

    #[test]
    fn test_regex_match_case_insensitive_empty_pattern() {
        assert!(regex_match_case_insensitive("anything", ""));
        assert!(regex_match_case_insensitive("", ""));
    }

    #[test]
    fn test_matches_any_pattern_single() {
        let patterns = vec!["clap".to_string()];
        assert!(matches_any_pattern("clap v4.5.54", &patterns));
        assert!(!matches_any_pattern("anyhow v1.0", &patterns));
    }

    #[test]
    fn test_matches_any_pattern_multiple() {
        let patterns = vec!["clap".to_string(), "anyhow".to_string()];
        assert!(matches_any_pattern("clap v4.5.54", &patterns));
        assert!(matches_any_pattern("anyhow v1.0", &patterns));
        assert!(!matches_any_pattern("regex v1.0", &patterns));
    }

    #[test]
    fn test_matches_any_pattern_empty() {
        let patterns: Vec<String> = vec![];
        assert!(!matches_any_pattern("clap v4.5", &patterns));
    }

    #[test]
    fn test_matches_any_pattern_case_insensitive() {
        let patterns = vec!["CLAP".to_string()];
        assert!(matches_any_pattern("clap v4.5", &patterns));
    }

    #[test]
    fn test_regex_match_case_insensitive_numbers() {
        assert!(regex_match_case_insensitive("clap v4.5.54", "4.5"));
        assert!(regex_match_case_insensitive("anyhow 1.0.0", "1.0"));
    }

    #[test]
    fn test_regex_match_case_insensitive_hyphens() {
        assert!(regex_match_case_insensitive(
            "test-package v1.0",
            "test-package"
        ));
        assert!(regex_match_case_insensitive("test-package v1.0", "test"));
        assert!(regex_match_case_insensitive("test-package v1.0", "package"));
    }

    #[test]
    fn test_matches_any_pattern_special_chars() {
        let patterns = vec!["test.pkg".to_string()];
        assert!(matches_any_pattern("test.pkg v1.0", &patterns));
        assert!(!matches_any_pattern("testXpkg v1.0", &patterns));
    }

    #[test]
    fn test_matches_any_pattern_three_patterns() {
        let patterns = vec![
            "clap".to_string(),
            "anyhow".to_string(),
            "regex".to_string(),
        ];
        assert!(matches_any_pattern("clap v4.5", &patterns));
        assert!(matches_any_pattern("anyhow v1.0", &patterns));
        assert!(matches_any_pattern("regex v1.10", &patterns));
        assert!(!matches_any_pattern("serde v1.0", &patterns));
    }

    #[test]
    fn test_parse_dependency_lines_basic() {
        let lines = vec![
            "clap v4.5.54".to_string(),
            "anyhow v1.0.95".to_string(),
        ];
        let result = parse_dependency_lines(&lines);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "clap");
        assert_eq!(result[0].version, Some("v4.5.54".to_string()));
        assert_eq!(result[1].name, "anyhow");
    }

    #[test]
    fn test_parse_dependency_lines_no_version() {
        let lines = vec!["some-package".to_string()];
        let result = parse_dependency_lines(&lines);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "some-package");
        assert_eq!(result[0].version, None);
    }

    #[test]
    fn test_parse_dependency_lines_empty() {
        let lines: Vec<String> = vec![];
        let result = parse_dependency_lines(&lines);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_dependency_lines_skips_blank() {
        let lines = vec!["".to_string(), "  ".to_string(), "clap v4.5".to_string()];
        let result = parse_dependency_lines(&lines);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "clap");
    }

    #[test]
    fn test_parse_dependency_lines_strips_tree_chars() {
        let lines = vec![
            "bashers v0.8.8 (/home/user/bashers)".to_string(),
            "├── anyhow v1.0.102".to_string(),
            "├── clap v4.5.54".to_string(),
            "└── regex v1.12.3".to_string(),
        ];
        let result = parse_dependency_lines(&lines);
        assert_eq!(result.len(), 4);
        assert_eq!(result[1].name, "anyhow");
        assert_eq!(result[1].version, Some("v1.0.102".to_string()));
        assert_eq!(result[2].name, "clap");
        assert_eq!(result[3].name, "regex");
    }

    #[test]
    fn test_parse_dependency_lines_strips_nested_tree_chars() {
        let lines = vec![
            "├── clap v4.5.54".to_string(),
            "│   └── clap-derive v4.5.54".to_string(),
        ];
        let result = parse_dependency_lines(&lines);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "clap");
        assert_eq!(result[1].name, "clap-derive");
    }

    #[test]
    fn test_parse_dependency_lines_skips_tree_only_lines() {
        let lines = vec![
            "│".to_string(),
            "│   ".to_string(),
            "├── clap v4.5".to_string(),
        ];
        let result = parse_dependency_lines(&lines);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "clap");
    }
}
