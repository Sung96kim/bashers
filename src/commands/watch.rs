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

fn run_cmd(program: &str, args: &[String]) -> Result<String> {
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
}
