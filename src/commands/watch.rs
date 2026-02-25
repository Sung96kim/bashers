use anyhow::{Context, Result};
use std::process::Command;
use std::time::Duration;

use crate::utils::colors::Colors;
use diff;

pub fn run(command: &[String], interval_secs: u64, no_diff: bool) -> Result<()> {
    if command.is_empty() {
        anyhow::bail!("command cannot be empty");
    }
    let program = &command[0];
    let args = &command[1..];

    ctrlc::set_handler(move || std::process::exit(0)).context("setting Ctrl+C handler")?;

    let mut colors = Colors::new();
    let mut previous: Option<String> = None;

    loop {
        let output = run_cmd(program, args)?;
        clear_screen();
        let show_diff = !no_diff && previous.is_some();
        print_header(interval_secs, command, &mut colors, show_diff)?;

        if no_diff {
            let _ = colors.reset();
            let _ = colors.println(&output);
        } else if let Some(ref prev) = previous {
            print_diff(prev, &output, &mut colors)?;
        } else {
            let _ = colors.reset();
            let _ = colors.println(&output);
        }
        previous = Some(output);

        let _ = colors.reset();
        let _ = colors.flush();
        std::thread::sleep(Duration::from_secs(interval_secs));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "gui", derive(serde::Serialize, serde::Deserialize))]
pub enum DiffSegment {
    Same(String),
    Added(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "gui", derive(serde::Serialize, serde::Deserialize))]
pub struct DiffLine {
    pub segments: Vec<DiffSegment>,
}

pub fn compute_diff_lines(prev: &str, curr: &str) -> Vec<DiffLine> {
    let results = diff::lines(prev, curr);
    let mut output = Vec::new();
    let mut pending_lefts: Vec<&str> = Vec::new();

    for r in results {
        match r {
            diff::Result::Left(line) => {
                pending_lefts.push(line);
            }
            diff::Result::Both(prev_line, curr_line) => {
                output.push(compute_char_diff(prev_line, curr_line));
            }
            diff::Result::Right(curr_line) => {
                if let Some(prev_line) = pending_lefts.pop() {
                    output.push(compute_char_diff(prev_line, curr_line));
                } else {
                    output.push(DiffLine {
                        segments: vec![DiffSegment::Added(curr_line.to_string())],
                    });
                }
            }
        }
    }
    output
}

fn compute_char_diff(prev_line: &str, curr_line: &str) -> DiffLine {
    let mut segments = Vec::new();
    let mut normal = String::new();
    let mut added = String::new();

    for r in diff::chars(prev_line, curr_line) {
        match r {
            diff::Result::Left(_) => {}
            diff::Result::Both(c, _) => {
                if !added.is_empty() {
                    segments.push(DiffSegment::Added(added.clone()));
                    added.clear();
                }
                normal.push(c);
            }
            diff::Result::Right(c) => {
                if !normal.is_empty() {
                    segments.push(DiffSegment::Same(normal.clone()));
                    normal.clear();
                }
                added.push(c);
            }
        }
    }
    if !normal.is_empty() {
        segments.push(DiffSegment::Same(normal));
    }
    if !added.is_empty() {
        segments.push(DiffSegment::Added(added));
    }
    DiffLine { segments }
}

pub fn run_cmd(program: &str, args: &[String]) -> Result<String> {
    let out = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("running {} {}", program, args.join(" ")))?;
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    if !out.stderr.is_empty() {
        s.push_str(&String::from_utf8_lossy(&out.stderr));
    }
    if s.ends_with('\n') {
        s.pop();
    }
    Ok(s)
}

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
}

fn print_header(
    interval_secs: u64,
    command: &[String],
    colors: &mut Colors,
    show_diff_hint: bool,
) -> std::io::Result<()> {
    let _ = colors.cyan();
    let _ = colors.bold();
    let _ = colors.print(&format!("Every {}s: ", interval_secs));
    let _ = colors.reset();
    let _ = colors.println(&command.join(" "));
    if show_diff_hint {
        let _ = colors.green();
        let _ = colors.print("green");
        let _ = colors.reset();
        let _ = colors.println(" = changed since last run");
    }
    let _ = colors.println("");
    Ok(())
}

fn print_diff(prev: &str, curr: &str, colors: &mut Colors) -> std::io::Result<()> {
    let results = diff::lines(prev, curr);
    let mut pending_lefts: Vec<&str> = Vec::new();
    for r in results {
        match r {
            diff::Result::Left(line) => {
                pending_lefts.push(line);
            }
            diff::Result::Both(prev_line, curr_line) => {
                print_line_char_diff(prev_line, curr_line, colors)?;
            }
            diff::Result::Right(curr_line) => {
                if let Some(prev_line) = pending_lefts.pop() {
                    print_line_char_diff(prev_line, curr_line, colors)?;
                } else {
                    let _ = colors.green();
                    let _ = colors.println(curr_line);
                }
            }
        }
    }
    Ok(())
}

fn print_line_char_diff(
    prev_line: &str,
    curr_line: &str,
    colors: &mut Colors,
) -> std::io::Result<()> {
    let mut normal = String::new();
    let mut green = String::new();
    for r in diff::chars(prev_line, curr_line) {
        match r {
            diff::Result::Left(_) => {}
            diff::Result::Both(c, _) => {
                if !green.is_empty() {
                    let _ = colors.green();
                    let _ = colors.print(&green);
                    let _ = colors.reset();
                    green.clear();
                }
                normal.push(c);
            }
            diff::Result::Right(c) => {
                if !normal.is_empty() {
                    let _ = colors.reset();
                    let _ = colors.print(&normal);
                    normal.clear();
                }
                green.push(c);
            }
        }
    }
    let _ = colors.reset();
    let _ = colors.print(&normal);
    let _ = colors.green();
    let _ = colors.print(&green);
    let _ = colors.reset();
    let _ = colors.println("");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_empty_command_errors() {
        let err = run(&[], 1, false).unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn test_run_cmd_captures_stdout() {
        let output = run_cmd("echo", &["hello".to_string()]).unwrap();
        assert_eq!(output, "hello");
    }

    #[test]
    fn test_run_cmd_strips_trailing_newline() {
        let output = run_cmd("printf", &["hello\n".to_string()]).unwrap();
        assert_eq!(output, "hello");
    }

    #[test]
    fn test_run_cmd_no_trailing_newline() {
        let output = run_cmd("printf", &["hello".to_string()]).unwrap();
        assert_eq!(output, "hello");
    }

    #[test]
    fn test_run_cmd_nonexistent_command_errors() {
        let result = run_cmd("nonexistent_command_12345", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_print_header_no_diff_hint() {
        let mut colors = Colors::new();
        let cmd = vec!["ls".to_string(), "-la".to_string()];
        let result = print_header(2, &cmd, &mut colors, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_header_with_diff_hint() {
        let mut colors = Colors::new();
        let cmd = vec!["ls".to_string()];
        let result = print_header(5, &cmd, &mut colors, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_diff_identical() {
        let mut colors = Colors::new();
        let result = print_diff("hello\nworld", "hello\nworld", &mut colors);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_diff_new_line_added() {
        let mut colors = Colors::new();
        let result = print_diff("hello", "hello\nnew line", &mut colors);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_diff_line_changed() {
        let mut colors = Colors::new();
        let result = print_diff("hello world", "hello earth", &mut colors);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_diff_empty_strings() {
        let mut colors = Colors::new();
        let result = print_diff("", "", &mut colors);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_line_char_diff_no_changes() {
        let mut colors = Colors::new();
        let result = print_line_char_diff("same", "same", &mut colors);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_line_char_diff_partial_change() {
        let mut colors = Colors::new();
        let result = print_line_char_diff("count: 5", "count: 10", &mut colors);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_cmd_with_args() {
        let output = run_cmd("printf", &["%s %s".to_string(), "a".to_string(), "b".to_string()]).unwrap();
        assert_eq!(output, "a b");
    }

    #[test]
    fn test_compute_diff_lines_identical() {
        let result = compute_diff_lines("hello\nworld", "hello\nworld");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].segments, vec![DiffSegment::Same("hello".to_string())]);
        assert_eq!(result[1].segments, vec![DiffSegment::Same("world".to_string())]);
    }

    #[test]
    fn test_compute_diff_lines_added_line() {
        let result = compute_diff_lines("hello", "hello\nnew");
        assert_eq!(result.len(), 2);
        assert_eq!(result[1].segments, vec![DiffSegment::Added("new".to_string())]);
    }

    #[test]
    fn test_compute_diff_lines_char_change() {
        let result = compute_diff_lines("count: 5", "count: 10");
        assert_eq!(result.len(), 1);
        assert!(result[0].segments.iter().any(|s| matches!(s, DiffSegment::Added(_))));
    }

    #[test]
    fn test_diff_segment_equality() {
        assert_eq!(DiffSegment::Same("a".into()), DiffSegment::Same("a".into()));
        assert_ne!(DiffSegment::Same("a".into()), DiffSegment::Added("a".into()));
    }
}
