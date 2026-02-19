use anyhow::{Context, Result};
use spinoff::{spinners, Color, Spinner, Streams};
use std::io::Write;
use std::process::{Command, ExitStatus};
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn create_spinner(msg: &str) -> Option<Spinner> {
    if !should_show_spinner() {
        return None;
    }
    let msg = colorize_spinner_message(msg, Color::Cyan);
    Some(Spinner::new_with_stream(
        spinners::Arrow2,
        msg,
        Color::Cyan,
        Streams::Stderr,
    ))
}

pub fn finish_with_message(sp: Option<&mut Spinner>, message: &str) {
    if let Some(sp) = sp {
        sp.clear();
        print_success_message_replace_line(message);
    }
}

pub fn stop_spinner(sp: Option<&mut Spinner>) {
    if let Some(sp) = sp {
        sp.stop();
    }
}

pub fn print_success_message(message: &str) {
    let mut stderr = StandardStream::stderr(if atty::is(atty::Stream::Stderr) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });
    let _ = stderr.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)));
    let _ = write!(stderr, "✓ {}\n", message);
    let _ = stderr.reset();
    let _ = stderr.flush();
}

pub fn print_success_message_replace_line(message: &str) {
    let mut stderr = StandardStream::stderr(if atty::is(atty::Stream::Stderr) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });
    let _ = write!(stderr, "\r\x1b[K");
    let _ = stderr.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)));
    let _ = write!(stderr, "✓ {}\n", message);
    let _ = stderr.reset();
    let _ = stderr.flush();
}

pub fn print_failure_message(message: &str) {
    let mut stderr = StandardStream::stderr(if atty::is(atty::Stream::Stderr) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });
    let _ = stderr.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Red)));
    let _ = write!(stderr, "✗ {}\n", message);
    let _ = stderr.reset();
    let _ = stderr.flush();
}

pub fn should_show_spinner() -> bool {
    if std::env::var("NO_SPINNER").is_ok() {
        return false;
    }
    atty::is(atty::Stream::Stdout)
}

fn colorize_spinner_message(msg: &str, color: Color) -> String {
    if !atty::is(atty::Stream::Stderr) {
        return msg.to_string();
    }
    let code = match color {
        Color::Red => "\x1b[31m",
        Color::Green => "\x1b[32m",
        Color::Yellow => "\x1b[33m",
        Color::Blue => "\x1b[34m",
        Color::Cyan => "\x1b[36m",
        Color::White | Color::Magenta | Color::Black | _ => "\x1b[0m",
    };
    format!("{}{}\x1b[0m", code, msg)
}

const DEFAULT_SUCCESS_MESSAGE: &str = "Updated";

pub fn run_with_completion<T, E>(
    dry_run: bool,
    spinner_msg: &str,
    success_msg: &str,
    color: Option<Color>,
    f: impl FnOnce() -> std::result::Result<T, E>,
    is_success: impl FnOnce(&T) -> bool,
) -> std::result::Result<T, E> {
    let show = !dry_run && should_show_spinner();
    let mut sp = if show {
        let color = color.unwrap_or(Color::Green);
        let msg = colorize_spinner_message(spinner_msg, color);
        Some(Spinner::new_with_stream(
            spinners::Arrow2,
            msg,
            color,
            Streams::Stderr,
        ))
    } else {
        None
    };
    let start = std::time::Instant::now();
    let result = f();
    if let Some(ref mut sp) = sp {
        const MIN_DISPLAY: std::time::Duration = std::time::Duration::from_millis(200);
        if let Some(remaining) = MIN_DISPLAY.checked_sub(start.elapsed()) {
            std::thread::sleep(remaining);
        }
        if let Ok(ref t) = result {
            if is_success(t) {
                sp.clear();
                print_success_message_replace_line(success_msg);
            } else {
                sp.clear();
            }
        } else {
            sp.clear();
        }
    }
    result
}

pub fn run_with_spinner(message: &str, command: &mut Command) -> Result<ExitStatus> {
    run_with_spinner_and_message(message, command, None)
}

pub fn run_with_spinner_and_message(
    message: &str,
    command: &mut Command,
    success_message: Option<&str>,
) -> Result<ExitStatus> {
    let mut sp = if should_show_spinner() {
        let msg = colorize_spinner_message(message, Color::Cyan);
        Some(Spinner::new_with_stream(
            spinners::Material,
            msg,
            Color::Cyan,
            Streams::Stderr,
        ))
    } else {
        None
    };

    let output = command.output().context("Failed to run command")?;
    let status = output.status;

    if let Some(ref mut sp) = sp {
        if status.success() {
            let msg = success_message.unwrap_or(DEFAULT_SUCCESS_MESSAGE);
            sp.clear();
            print_success_message_replace_line(msg);
        } else {
            sp.clear();
        }
    }

    let _ = std::io::stdout().write_all(&output.stdout);
    let _ = std::io::stderr().write_all(&output.stderr);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();

    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_should_show_spinner() {
        let _: bool = should_show_spinner();
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
        let _: bool = should_show_spinner();
    }

    #[test]
    fn test_run_with_completion_dry_run_success() {
        let out: Result<i32, ()> =
            run_with_completion(true, "msg", "done", None, || Ok(42), |&x| x > 0);
        assert!(out.is_ok());
        assert_eq!(out.unwrap(), 42);
    }

    #[test]
    fn test_run_with_completion_dry_run_failure() {
        let out: Result<i32, ()> =
            run_with_completion(true, "msg", "done", None, || Ok(0), |&x| x > 0);
        assert!(out.is_ok());
        assert_eq!(out.unwrap(), 0);
    }

    #[test]
    fn test_run_with_completion_dry_run_err() {
        let out: Result<i32, &str> = run_with_completion(
            true,
            "msg",
            "done",
            None,
            || Err("error"),
            |&x: &i32| x > 0,
        );
        assert!(out.is_err());
        assert_eq!(out.unwrap_err(), "error");
    }

    #[test]
    fn test_run_with_completion_no_spinner_success() {
        std::env::set_var("NO_SPINNER", "1");
        let out: Result<i32, ()> =
            run_with_completion(false, "msg", "done", None, || Ok(1), |&x| x > 0);
        std::env::remove_var("NO_SPINNER");
        assert!(out.is_ok());
        assert_eq!(out.unwrap(), 1);
    }

    #[test]
    fn test_run_with_completion_no_spinner_err() {
        std::env::set_var("NO_SPINNER", "1");
        let out: Result<i32, &str> = run_with_completion(
            false,
            "msg",
            "done",
            None,
            || Err("fail"),
            |&x: &i32| x > 0,
        );
        std::env::remove_var("NO_SPINNER");
        assert!(out.is_err());
    }

    #[test]
    fn test_print_success_message_no_panic() {
        print_success_message("test");
    }

    #[test]
    fn test_print_success_message_replace_line_no_panic() {
        print_success_message_replace_line("test");
    }

    #[test]
    fn test_print_failure_message_no_panic() {
        print_failure_message("test");
    }

    #[test]
    fn test_finish_with_message_none_no_panic() {
        finish_with_message(None, "msg");
    }

    #[test]
    fn test_stop_spinner_none_no_panic() {
        stop_spinner(None);
    }

    #[test]
    fn test_create_spinner_returns_none_when_no_spinner() {
        std::env::set_var("NO_SPINNER", "1");
        let sp = create_spinner("loading");
        std::env::remove_var("NO_SPINNER");
        assert!(sp.is_none());
    }

    #[test]
    fn test_run_with_spinner_and_message_success() {
        std::env::set_var("NO_SPINNER", "1");
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(["/c", "exit 0"]);
            c
        } else {
            Command::new("true")
        };
        let result = run_with_spinner_and_message("running", &mut cmd, Some("Done"));
        std::env::remove_var("NO_SPINNER");
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn test_run_with_spinner_and_message_failure() {
        std::env::set_var("NO_SPINNER", "1");
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(["/c", "exit 1"]);
            c
        } else {
            Command::new("false")
        };
        let result = run_with_spinner_and_message("running", &mut cmd, None);
        std::env::remove_var("NO_SPINNER");
        assert!(result.is_ok());
        assert!(!result.unwrap().success());
    }
}
