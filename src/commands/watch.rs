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
        print_header(interval_secs, command, &mut colors)?;

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

fn print_header(interval_secs: u64, command: &[String], colors: &mut Colors) -> std::io::Result<()> {
    let _ = colors.cyan();
    let _ = colors.bold();
    let _ = colors.print(&format!("Every {}s: ", interval_secs));
    let _ = colors.reset();
    let _ = colors.println(&command.join(" "));
    let _ = colors.println("");
    Ok(())
}

fn print_diff(prev: &str, curr: &str, colors: &mut Colors) -> std::io::Result<()> {
    let results = diff::lines(prev, curr);
    for r in results {
        match r {
            diff::Result::Left(line) => {
                let _ = colors.red();
                let _ = colors.print("- ");
                let _ = colors.println(line);
            }
            diff::Result::Both(line, _) => {
                let _ = colors.reset();
                let _ = colors.print("  ");
                let _ = colors.println(line);
            }
            diff::Result::Right(line) => {
                let _ = colors.green();
                let _ = colors.print("+ ");
                let _ = colors.println(line);
            }
        }
    }
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
