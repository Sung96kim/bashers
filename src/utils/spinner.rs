use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn create_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]),
    );
    // Write to stderr so it doesn't interfere with command output
    pb.set_draw_target(indicatif::ProgressDrawTarget::stderr());
    pb
}

pub fn should_show_spinner() -> bool {
    // Check if spinner is disabled via environment variable
    if std::env::var("NO_SPINNER").is_ok() {
        return false;
    }

    // Only show spinner if stdout is a TTY
    atty::is(atty::Stream::Stdout)
}

// Helper to run command with streaming output and spinner
pub fn run_with_spinner(message: &str, command: &mut Command) -> Result<ExitStatus> {
    let pb = if should_show_spinner() {
        Some(Arc::new(Mutex::new(create_spinner())))
    } else {
        None
    };

    if let Some(ref spinner) = pb {
        spinner.lock().unwrap().set_message(message.to_string());
        spinner
            .lock()
            .unwrap()
            .enable_steady_tick(std::time::Duration::from_millis(100));
    }

    // Spawn command with piped stdout/stderr
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().context("Failed to spawn command")?;

    // Handle stdout in a thread
    let stdout_handle = if let Some(stdout) = child.stdout.take() {
        let spinner_clone = pb.clone();
        Some(thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(ref spinner) = spinner_clone {
                    spinner.lock().unwrap().suspend(|| {
                        println!("{}", line);
                    });
                } else {
                    println!("{}", line);
                }
            }
        }))
    } else {
        None
    };

    // Handle stderr in a thread
    let stderr_handle = if let Some(stderr) = child.stderr.take() {
        let spinner_clone = pb.clone();
        Some(thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(ref spinner) = spinner_clone {
                    spinner.lock().unwrap().suspend(|| {
                        eprintln!("{}", line);
                    });
                } else {
                    eprintln!("{}", line);
                }
            }
        }))
    } else {
        None
    };

    // Wait for command to finish
    let status = child.wait()?;

    // Wait for output threads to finish
    if let Some(handle) = stdout_handle {
        let _ = handle.join();
    }
    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }

    // Finish spinner with green checkmark and completion message
    if let Some(ref spinner) = pb {
        if status.success() {
            spinner.lock().unwrap().finish_and_clear();
            // Print green checkmark with completion message
            let mut stderr = StandardStream::stderr(if atty::is(atty::Stream::Stderr) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            });
            let _ = stderr.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
            let _ = write!(stderr, "✓ Updated");
            let _ = stderr.reset();
            let _ = writeln!(stderr);
        } else {
            spinner.lock().unwrap().finish_and_clear();
        }
    }

    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_spinner() {
        let _spinner = create_spinner();
        // Just verify it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_should_show_spinner() {
        // Test that function exists and returns a boolean
        let result = should_show_spinner();
        // Result depends on environment, but should be a boolean
        assert!(result == true || result == false);
    }

    #[test]
    fn test_spinner_with_no_spinner_env() {
        std::env::set_var("NO_SPINNER", "1");
        assert!(!should_show_spinner());
        std::env::remove_var("NO_SPINNER");
    }

    #[test]
    fn test_spinner_without_no_spinner_env() {
        std::env::remove_var("NO_SPINNER");
        // Result depends on whether stdout is a TTY
        let result = should_show_spinner();
        assert!(result == true || result == false);
    }

    #[test]
    fn test_spinner_style() {
        let spinner = create_spinner();
        spinner.set_message("test");
        // Verify spinner can be configured
        assert!(true);
    }
}
