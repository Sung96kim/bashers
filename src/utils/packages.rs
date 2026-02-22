use crate::utils::project::ProjectType;
use anyhow::{Context, Result};
use std::process::Command;

pub fn list(project_type: ProjectType) -> Result<Vec<String>> {
    match project_type {
        ProjectType::Uv => list_uv(),
        ProjectType::Poetry => list_poetry(),
        ProjectType::Cargo => list_cargo(),
    }
}

fn list_uv() -> Result<Vec<String>> {
    let output = Command::new("uv")
        .args(["pip", "list"])
        .output()
        .context("Failed to run uv pip list")?;

    if !output.status.success() {
        anyhow::bail!("uv pip list failed");
    }

    let stdout = String::from_utf8(output.stdout)?;
    let packages: Vec<String> = stdout
        .lines()
        .skip(2)
        .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
        .collect();

    Ok(packages)
}

fn list_poetry() -> Result<Vec<String>> {
    let output = Command::new("poetry")
        .arg("show")
        .output()
        .context("Failed to run poetry show")?;

    if !output.status.success() {
        anyhow::bail!("poetry show failed");
    }

    let stdout = String::from_utf8(output.stdout)?;
    let packages: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            if line.starts_with("name") {
                line.split(':').nth(1).map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(packages)
}

fn list_cargo() -> Result<Vec<String>> {
    let output = Command::new("cargo")
        .args(["tree", "--depth", "1", "--format", "{p}"])
        .output()
        .context("Failed to run cargo tree")?;

    if !output.status.success() {
        anyhow::bail!("cargo tree failed");
    }

    let stdout = String::from_utf8(output.stdout)?;
    let mut packages = Vec::new();

    for line in stdout.lines() {
        // cargo tree format: "├── package_name vX.Y.Z" or "└── package_name vX.Y.Z"
        // or just "package_name vX.Y.Z" for root
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Remove tree characters
        let line = line.trim_start_matches("├── ").trim_start_matches("└── ");

        // Extract package name (everything before the version)
        if let Some(name_end) = line.find(' ') {
            let name = line[..name_end].trim();
            if !name.is_empty() && name != "bashers" {
                packages.push(name.to_string());
            }
        }
    }

    Ok(packages)
}

pub fn get_installed_version(project_type: ProjectType, package: &str) -> Result<Option<String>> {
    match project_type {
        ProjectType::Uv => get_version_uv(package),
        ProjectType::Poetry => get_version_poetry(package),
        ProjectType::Cargo => get_version_cargo(package),
    }
}

fn get_version_uv(package: &str) -> Result<Option<String>> {
    let output = Command::new("uv")
        .args(["pip", "show", package])
        .output()
        .context("Failed to run uv pip show")?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8(output.stdout)?;
    for line in stdout.lines() {
        if let Some(v) = line.strip_prefix("Version:") {
            return Ok(Some(v.trim().to_string()));
        }
    }
    Ok(None)
}

fn get_version_poetry(package: &str) -> Result<Option<String>> {
    let output = Command::new("poetry")
        .args(["show", package])
        .output()
        .context("Failed to run poetry show")?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8(output.stdout)?;
    for line in stdout.lines() {
        if let Some(v) = line.strip_prefix("version") {
            let v = v.trim_start_matches(|c| c == ' ' || c == ':');
            if !v.is_empty() {
                return Ok(Some(v.trim().to_string()));
            }
        }
    }
    Ok(None)
}

fn get_version_cargo(package: &str) -> Result<Option<String>> {
    let output = Command::new("cargo")
        .args(["tree", "-p", package, "--depth", "0"])
        .output()
        .context("Failed to run cargo tree")?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8(output.stdout)?;
    for line in stdout.lines() {
        let line = line
            .trim()
            .trim_start_matches("├── ")
            .trim_start_matches("└── ");
        if let Some(rest) = line.strip_prefix(package) {
            let rest = rest.trim();
            if let Some(version) = rest.strip_prefix('v') {
                return Ok(Some(version.to_string()));
            }
            if !rest.is_empty() && rest.chars().next().map(|c| c.is_ascii_digit()) == Some(true) {
                return Ok(Some(rest.to_string()));
            }
        }
    }
    Ok(None)
}

pub fn fuzzy_match(packages: &[String], pattern: &str) -> Result<Vec<String>> {
    use fuzzy_matcher::skim::SkimMatcherV2;
    use fuzzy_matcher::FuzzyMatcher;

    let matcher = SkimMatcherV2::default();
    let mut matches: Vec<(i64, String)> = packages
        .iter()
        .filter_map(|pkg| {
            matcher
                .fuzzy_match(pkg, pattern)
                .map(|score| (score, pkg.clone()))
        })
        .collect();

    matches.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(matches.into_iter().map(|(_, pkg)| pkg).collect())
}

pub fn select_one(matches: Vec<String>) -> Result<String> {
    if matches.is_empty() {
        anyhow::bail!("No packages found");
    }

    if matches.len() == 1 {
        return Ok(matches[0].clone());
    }

    if atty::is(atty::Stream::Stdin) {
        return select_with_inquire(&matches);
    }

    eprintln!("Multiple packages found:");
    for pkg in &matches {
        eprintln!("  {}", pkg);
    }
    eprintln!("Non-interactive terminal - cannot select");
    anyhow::bail!("Multiple matches found - interactive selection required");
}

pub fn select_one_with_auto_select(matches: Vec<String>, auto_select: bool) -> Result<String> {
    if matches.is_empty() {
        anyhow::bail!("No packages found");
    }

    if matches.len() == 1 {
        return Ok(matches[0].clone());
    }

    if auto_select {
        eprintln!("Multiple packages found, selecting first match:");
        for pkg in &matches {
            eprintln!("  {}", pkg);
        }
        eprintln!("Selected: {}", matches[0]);
        return Ok(matches[0].clone());
    }

    if atty::is(atty::Stream::Stdin) {
        return select_with_inquire(&matches);
    }

    eprintln!("Multiple packages found:");
    for pkg in &matches {
        eprintln!("  {}", pkg);
    }
    eprintln!("Non-interactive terminal - cannot select");
    anyhow::bail!("Multiple matches found");
}

fn select_with_inquire(matches: &[String]) -> Result<String> {
    use inquire::Select;

    let selected = Select::new("Select a package:", matches.to_vec())
        .with_page_size(10)
        .prompt()
        .context("Failed to select package")?;

    Ok(selected)
}

pub fn select_many(matches: Vec<String>) -> Result<Vec<String>> {
    if matches.is_empty() {
        anyhow::bail!("No packages found");
    }

    if matches.len() == 1 {
        return Ok(matches);
    }

    if atty::is(atty::Stream::Stdin) {
        return select_many_with_inquire(&matches);
    }

    eprintln!("Multiple packages found (multi-select requires interactive terminal):");
    for pkg in &matches {
        eprintln!("  {}", pkg);
    }
    anyhow::bail!("Multi-select requires interactive selection");
}

pub fn select_many_with_auto_select(
    matches: Vec<String>,
    auto_select: bool,
) -> Result<Vec<String>> {
    if matches.is_empty() {
        anyhow::bail!("No packages found");
    }

    if matches.len() == 1 {
        return Ok(matches);
    }

    if auto_select {
        eprintln!("Selecting all {} matching packages:", matches.len());
        for pkg in &matches {
            eprintln!("  {}", pkg);
        }
        return Ok(matches);
    }

    if atty::is(atty::Stream::Stdin) {
        return select_many_with_inquire(&matches);
    }

    eprintln!("Multiple packages found (multi-select requires interactive terminal):");
    for pkg in &matches {
        eprintln!("  {}", pkg);
    }
    anyhow::bail!("Multi-select requires interactive selection");
}

fn select_many_with_inquire(matches: &[String]) -> Result<Vec<String>> {
    use inquire::list_option::ListOption;
    use inquire::MultiSelect;

    let formatter = |opts: &[ListOption<&String>]| {
        let names: Vec<&str> = opts.iter().map(|o| o.value.as_str()).collect();
        format!("Selected: {}", names.join(", "))
    };

    let selected = MultiSelect::new(
        "Select packages (space to toggle, enter to confirm):",
        matches.to_vec(),
    )
    .with_formatter(&formatter)
    .with_page_size(10)
    .prompt()
    .context("Failed to select packages")?;

    Ok(selected)
}

// Helper function to parse cargo tree output (extracted for testing)
#[cfg(test)]
fn parse_cargo_tree_output(output: &str) -> Vec<String> {
    let mut packages = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let line = line.trim_start_matches("├── ").trim_start_matches("└── ");

        if let Some(name_end) = line.find(' ') {
            let name = line[..name_end].trim();
            if !name.is_empty() && name != "bashers" {
                packages.push(name.to_string());
            }
        }
    }

    packages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match_exact() {
        let packages = vec![
            "clap".to_string(),
            "anyhow".to_string(),
            "regex".to_string(),
        ];
        let matches = fuzzy_match(&packages, "clap").unwrap();
        assert_eq!(matches, vec!["clap"]);
    }

    #[test]
    fn test_fuzzy_match_partial() {
        let packages = vec![
            "clap".to_string(),
            "clap-derive".to_string(),
            "anyhow".to_string(),
        ];
        let matches = fuzzy_match(&packages, "clap").unwrap();
        assert!(matches.contains(&"clap".to_string()));
        assert!(matches.contains(&"clap-derive".to_string()));
    }

    #[test]
    fn test_fuzzy_match_no_matches() {
        let packages = vec!["clap".to_string(), "anyhow".to_string()];
        let matches = fuzzy_match(&packages, "nonexistent").unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        let packages = vec!["Clap".to_string(), "ANYHOW".to_string()];
        let matches = fuzzy_match(&packages, "clap").unwrap();
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_fuzzy_match_empty_pattern() {
        let packages = vec!["clap".to_string(), "anyhow".to_string()];
        let matches = fuzzy_match(&packages, "").unwrap();
        // Empty pattern should match all or none depending on matcher behavior
        assert!(matches.len() <= packages.len());
    }

    #[test]
    fn test_fuzzy_match_ordering() {
        let packages = vec![
            "clap".to_string(),
            "clap-derive".to_string(),
            "clap-utils".to_string(),
        ];
        let matches = fuzzy_match(&packages, "clap").unwrap();
        // Exact match should come first
        assert_eq!(matches[0], "clap");
    }

    #[test]
    fn test_fuzzy_match_single_char() {
        let packages = vec![
            "clap".to_string(),
            "anyhow".to_string(),
            "regex".to_string(),
        ];
        let matches = fuzzy_match(&packages, "c").unwrap();
        assert!(matches.contains(&"clap".to_string()));
    }

    #[test]
    fn test_fuzzy_match_multiple_words() {
        let packages = vec!["clap-derive".to_string(), "clap-utils".to_string()];
        let matches = fuzzy_match(&packages, "clap derive").unwrap();
        // Fuzzy matching with spaces may or may not work depending on matcher
        // Just verify it doesn't panic and returns reasonable results
        assert!(matches.len() <= packages.len());
    }

    #[test]
    fn test_select_one_single_match() {
        // Single match should always succeed
        let matches = vec!["clap".to_string()];
        let result = select_one(matches.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "clap");

        let result2 = select_one_with_auto_select(matches, false);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), "clap");
    }

    #[test]
    fn test_select_one_no_matches() {
        // Empty matches should fail
        let matches = vec![];
        let result = select_one(matches.clone());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No packages found"));

        let result2 = select_one_with_auto_select(matches, true);
        assert!(result2.is_err());
    }

    #[test]
    fn test_select_one_multiple_matches_non_interactive() {
        // When stdin is not a TTY with multiple matches,
        // select_one should fail (non-interactive)
        let matches = vec!["clap".to_string(), "anyhow".to_string()];
        let result = select_one(matches);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Multiple matches found"));
    }

    #[test]
    fn test_select_one_with_auto_select() {
        // When auto_select is true, it should select the first match
        let matches = vec!["clap".to_string(), "anyhow".to_string()];
        let result = select_one_with_auto_select(matches, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "clap");
    }

    #[test]
    fn test_select_one_with_auto_select_false() {
        // When auto_select is false and non-interactive, should fail
        let matches = vec!["clap".to_string(), "anyhow".to_string()];
        let result = select_one_with_auto_select(matches, false);
        // In non-interactive environment, this should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_select_many_empty() {
        let result = select_many(vec![]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No packages found"));
    }

    #[test]
    fn test_select_many_single() {
        let matches = vec!["clap".to_string()];
        let result = select_many(matches.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["clap".to_string()]);
    }

    #[test]
    fn test_select_many_multiple_non_interactive() {
        let matches = vec!["clap".to_string(), "anyhow".to_string()];
        let result = select_many(matches);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("interactive"));
    }

    #[test]
    fn test_select_many_with_auto_select() {
        let matches = vec!["clap".to_string(), "anyhow".to_string()];
        let result = select_many_with_auto_select(matches.clone(), true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), matches);
    }

    #[test]
    fn test_select_many_with_auto_select_single() {
        let matches = vec!["clap".to_string()];
        let result = select_many_with_auto_select(matches.clone(), true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), matches);
    }

    #[test]
    fn test_parse_cargo_tree_output() {
        let output = "bashers v0.4.9 (/home/sung9/bashers)
├── anyhow v1.0.100
├── clap v4.5.54
└── regex v1.12.2";

        let packages = parse_cargo_tree_output(output);

        assert_eq!(packages.len(), 3);
        assert!(packages.contains(&"anyhow".to_string()));
        assert!(packages.contains(&"clap".to_string()));
        assert!(packages.contains(&"regex".to_string()));
    }

    #[test]
    fn test_parse_cargo_tree_output_no_dependencies() {
        let output = "bashers v0.4.9 (/home/sung9/bashers)";

        let packages = parse_cargo_tree_output(output);

        assert_eq!(packages.len(), 0);
    }

    #[test]
    fn test_parse_cargo_tree_output_various_formats() {
        let output = "bashers v0.4.9
├── pkg1 v1.0.0
│   └── subpkg v0.1.0
└── pkg2 v2.0.0";

        let packages = parse_cargo_tree_output(output);

        assert!(packages.contains(&"pkg1".to_string()));
        assert!(packages.contains(&"pkg2".to_string()));
    }

    #[test]
    fn test_parse_cargo_tree_output_with_root() {
        let output = "bashers v0.4.9
├── anyhow v1.0.100
└── clap v4.5.54";

        let packages = parse_cargo_tree_output(output);

        // Should not include "bashers"
        assert!(!packages.contains(&"bashers".to_string()));
        assert!(packages.contains(&"anyhow".to_string()));
        assert!(packages.contains(&"clap".to_string()));
    }

    #[test]
    fn test_parse_cargo_tree_output_empty_lines() {
        let output = "bashers v0.4.9

├── pkg1 v1.0.0

└── pkg2 v2.0.0
";

        let packages = parse_cargo_tree_output(output);

        assert_eq!(packages.len(), 2);
        assert!(packages.contains(&"pkg1".to_string()));
        assert!(packages.contains(&"pkg2".to_string()));
    }

    #[test]
    fn test_parse_cargo_tree_output_whitespace() {
        let output = "bashers v0.4.9
├── pkg1 v1.0.0
└── pkg2 v2.0.0";

        let packages = parse_cargo_tree_output(output);

        assert_eq!(packages.len(), 2);
        assert!(packages.contains(&"pkg1".to_string()));
        assert!(packages.contains(&"pkg2".to_string()));
    }

    #[test]
    fn test_list_function_all_types() {
        let _ = list(ProjectType::Uv);
        let _ = list(ProjectType::Poetry);
        let _ = list(ProjectType::Cargo);
    }

    #[test]
    fn test_fuzzy_match_unicode() {
        let packages = vec!["café".to_string(), "naïve".to_string()];
        let matches = fuzzy_match(&packages, "cafe").unwrap();
        // Unicode matching depends on fuzzy matcher - may or may not match
        // Just verify it doesn't panic
        assert!(matches.len() <= packages.len());
    }

    #[test]
    fn test_fuzzy_match_special_chars() {
        let packages = vec!["test-package".to_string(), "test_package".to_string()];
        let matches = fuzzy_match(&packages, "test").unwrap();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_fuzzy_match_empty_packages() {
        let packages = vec![];
        let matches = fuzzy_match(&packages, "anything").unwrap();
        assert!(matches.is_empty());
    }
}
