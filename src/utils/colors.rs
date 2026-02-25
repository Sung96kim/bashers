use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub const ANSI_CYAN_BOLD: &str = "\x1b[36m\x1b[1m";
pub const ANSI_GREEN: &str = "\x1b[32m";
pub const ANSI_RED: &str = "\x1b[31m";
pub const ANSI_YELLOW: &str = "\x1b[33m";
pub const ANSI_DIM: &str = "\x1b[2m";
pub const ANSI_RESET: &str = "\x1b[0m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionChange {
    Upgraded,
    Unchanged,
    Downgraded,
}

pub fn format_bumped_message_colored(before: &str, after: &str, change: VersionChange) -> String {
    if atty::is(atty::Stream::Stderr) {
        let after_color = match change {
            VersionChange::Upgraded => ANSI_GREEN,
            VersionChange::Unchanged => ANSI_DIM,
            VersionChange::Downgraded => ANSI_RED,
        };
        format!(
            "bumped from {}{}{} -> {}{}{}",
            ANSI_YELLOW, before, ANSI_RESET, after_color, after, ANSI_RESET
        )
    } else {
        format!("bumped from {} -> {}", before, after)
    }
}

pub struct Colors {
    stdout: StandardStream,
}

impl Default for Colors {
    fn default() -> Self {
        Self::new()
    }
}

impl Colors {
    pub fn new() -> Self {
        let choice = if atty::is(atty::Stream::Stdout) {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };
        Self {
            stdout: StandardStream::stdout(choice),
        }
    }

    pub fn green(&mut self) -> io::Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
    }

    pub fn cyan(&mut self) -> io::Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
    }

    pub fn bold(&mut self) -> io::Result<()> {
        self.stdout.set_color(ColorSpec::new().set_bold(true))
    }

    pub fn reset(&mut self) -> io::Result<()> {
        self.stdout.reset()
    }

    pub fn red(&mut self) -> io::Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
    }

    pub fn yellow(&mut self) -> io::Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
    }

    pub fn print(&mut self, text: &str) -> io::Result<()> {
        write!(&mut self.stdout, "{}", text)
    }

    pub fn println(&mut self, text: &str) -> io::Result<()> {
        writeln!(&mut self.stdout, "{}", text)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}

pub fn print_update(package: &str) {
    let mut colors = Colors::new();
    let _ = colors.green();
    let _ = colors.print("[update]");
    let _ = colors.reset();
    let _ = colors.print(": updating ");
    let _ = colors.green();
    let _ = colors.print(package);
    let _ = colors.reset();
    let _ = colors.println("");
}

pub fn print_updated_version(package: &str, version: &str) {
    let v = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{}", version)
    };
    let mut colors = Colors::new();
    let _ = colors.green();
    let _ = colors.print("[update]");
    let _ = colors.reset();
    let _ = colors.print(": ");
    let _ = colors.green();
    let _ = colors.print(package);
    let _ = colors.reset();
    let _ = colors.println(&format!(" is now {}", v));
}

pub fn print_bumped_version(package: &str, before: &str, after: &str) {
    let mut colors = Colors::new();
    let _ = colors.green();
    let _ = colors.print("[update]");
    let _ = colors.reset();
    let _ = colors.print(": ");
    let _ = colors.green();
    let _ = colors.print(package);
    let _ = colors.reset();
    let _ = colors.print(" bumped from ");
    let _ = colors.yellow();
    let _ = colors.print(before);
    let _ = colors.reset();
    let _ = colors.print(" -> ");
    let _ = colors.green();
    let _ = colors.print(after);
    let _ = colors.reset();
    let _ = colors.println("");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colors_new() {
        let _colors = Colors::new();
    }

    #[test]
    fn test_colors_methods() {
        let mut colors = Colors::new();

        // Test all color methods don't panic
        assert!(colors.green().is_ok());
        assert!(colors.cyan().is_ok());
        assert!(colors.bold().is_ok());
        assert!(colors.red().is_ok());
        assert!(colors.yellow().is_ok());
        assert!(colors.reset().is_ok());
    }

    #[test]
    fn test_colors_print() {
        let mut colors = Colors::new();
        assert!(colors.print("test").is_ok());
        assert!(colors.println("test").is_ok());
    }

    #[test]
    fn test_print_update() {
        print_update("test-package");
    }

    #[test]
    fn test_colors_sequence() {
        let mut colors = Colors::new();
        assert!(colors.green().is_ok());
        assert!(colors.print("green text").is_ok());
        assert!(colors.reset().is_ok());
        assert!(colors.cyan().is_ok());
        assert!(colors.bold().is_ok());
        assert!(colors.println("bold cyan").is_ok());
        assert!(colors.reset().is_ok());
    }

    #[test]
    fn test_format_bumped_message_colored_contains_versions() {
        for change in [
            VersionChange::Upgraded,
            VersionChange::Unchanged,
            VersionChange::Downgraded,
        ] {
            let s = format_bumped_message_colored("v1.0.0", "v1.0.102", change);
            assert!(s.contains("bumped from"));
            assert!(s.contains("1.0.0"));
            assert!(s.contains("1.0.102"));
            assert!(s.contains(" -> "));
        }
    }

    #[test]
    fn test_version_change_equality() {
        assert_eq!(VersionChange::Upgraded, VersionChange::Upgraded);
        assert_ne!(VersionChange::Upgraded, VersionChange::Downgraded);
    }

    #[test]
    fn test_print_updated_version_no_panic() {
        print_updated_version("clap", "4.5.0");
    }

    #[test]
    fn test_print_updated_version_with_v_prefix() {
        print_updated_version("clap", "v4.5.0");
    }

    #[test]
    fn test_print_bumped_version_no_panic() {
        print_bumped_version("clap", "4.4.0", "4.5.0");
    }

    #[test]
    fn test_colors_flush() {
        let mut colors = Colors::new();
        assert!(colors.flush().is_ok());
    }

    #[test]
    fn test_colors_default() {
        let colors = Colors::default();
        let _ = colors;
    }

    #[test]
    fn test_version_change_all_variants() {
        let variants = [
            VersionChange::Upgraded,
            VersionChange::Unchanged,
            VersionChange::Downgraded,
        ];
        for v in &variants {
            let cloned = *v;
            assert_eq!(*v, cloned);
        }
    }

    #[test]
    fn test_ansi_constants_are_valid_escapes() {
        assert!(ANSI_CYAN_BOLD.starts_with("\x1b["));
        assert!(ANSI_GREEN.starts_with("\x1b["));
        assert!(ANSI_RED.starts_with("\x1b["));
        assert!(ANSI_YELLOW.starts_with("\x1b["));
        assert!(ANSI_DIM.starts_with("\x1b["));
        assert_eq!(ANSI_RESET, "\x1b[0m");
    }
}
