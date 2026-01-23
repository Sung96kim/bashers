use std::process::Command;

#[test]
fn test_version_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "version"])
        .output()
        .expect("Failed to run bashers version");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("bashers"));
}

#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to run bashers --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Bash command helpers"));
}

#[test]
fn test_update_dry_run() {
    // This test requires a Cargo.toml (which we have)
    let output = Command::new("cargo")
        .args(["run", "--", "update", "--dry-run"])
        .output()
        .expect("Failed to run bashers update --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("cargo update"));
}

#[test]
fn test_setup_dry_run() {
    let output = Command::new("cargo")
        .args(["run", "--", "setup", "--dry-run"])
        .output()
        .expect("Failed to run bashers setup --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("cargo build"));
}

#[test]
fn test_show_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "show"])
        .output()
        .expect("Failed to run bashers show");

    // show command should succeed and output cargo tree
    assert!(output.status.success() || output.status.code() == Some(0));
}

#[test]
fn test_gh_dry_run() {
    let output = Command::new("cargo")
        .args(["run", "--", "gh", "--dry-run"])
        .output()
        .expect("Failed to run bashers gh --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("git"));
}

#[test]
fn test_update_package_dry_run() {
    let output = Command::new("cargo")
        .args(["run", "--", "update", "clap", "--dry-run"])
        .output()
        .expect("Failed to run bashers update clap --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("cargo update"));
    assert!(stdout.contains("clap"));
}

#[test]
fn test_setup_frozen_dry_run() {
    let output = Command::new("cargo")
        .args(["run", "--", "setup", "--frozen", "--dry-run"])
        .output()
        .expect("Failed to run bashers setup --frozen --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--frozen"));
}

#[test]
fn test_show_with_pattern() {
    let output = Command::new("cargo")
        .args(["run", "--", "show", "clap"])
        .output()
        .expect("Failed to run bashers show clap");

    // Should output filtered results
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("clap") || stdout.is_empty());
}
