use crate::cli::BashersApp;
use anyhow::Result;
use clap::CommandFactory;

pub fn run() -> Result<()> {
    BashersApp::command().print_help()?;
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
