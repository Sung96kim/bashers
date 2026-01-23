use crate::utils::colors::Colors;
use anyhow::Result;

pub fn run() -> Result<()> {
    let mut colors = Colors::new();

    let _ = colors.bold();
    let _ = colors.println("Bashers - Bash command helpers");
    let _ = colors.reset();
    let _ = colors.println("");

    let _ = colors.bold();
    let _ = colors.print("Usage: ");
    let _ = colors.reset();
    let _ = colors.println("bashers <COMMAND> [ARGS]");
    let _ = colors.println("");

    let _ = colors.bold();
    let _ = colors.println("Commands:");
    let _ = colors.reset();

    let _ = colors.cyan();
    let _ = colors.print("  update");
    let _ = colors.reset();
    let _ = colors.println("    Update Python dependencies (uv/poetry)");

    let _ = colors.cyan();
    let _ = colors.print("  setup");
    let _ = colors.reset();
    let _ = colors.println("    Install project dependencies (uv/poetry)");

    let _ = colors.cyan();
    let _ = colors.print("  show");
    let _ = colors.reset();
    let _ = colors.println("    List installed packages (uv/poetry)");

    let _ = colors.cyan();
    let _ = colors.print("  gh");
    let _ = colors.reset();
    let _ = colors.println("    Git home: checkout default branch, pull, fetch all");

    let _ = colors.println("");
    let _ = colors.bold();
    let _ = colors.print("Use ");
    let _ = colors.reset();
    let _ = colors.bold();
    let _ = colors.print("bashers <command> --help");
    let _ = colors.reset();
    let _ = colors.println(" for more details.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_run() {
        // Test that help command runs without error
        let result = run();
        assert!(result.is_ok());
    }
}
