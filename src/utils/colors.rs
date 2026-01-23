use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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

    pub fn print(&mut self, text: &str) -> io::Result<()> {
        write!(&mut self.stdout, "{}", text)
    }

    pub fn println(&mut self, text: &str) -> io::Result<()> {
        writeln!(&mut self.stdout, "{}", text)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colors_new() {
        let _colors = Colors::new();
        // Just verify it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_colors_methods() {
        let mut colors = Colors::new();

        // Test all color methods don't panic
        assert!(colors.green().is_ok());
        assert!(colors.cyan().is_ok());
        assert!(colors.bold().is_ok());
        assert!(colors.red().is_ok());
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
        // Should not panic
        print_update("test-package");
        assert!(true);
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
}
