use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bashers")]
#[command(about = "Bash command helpers", long_about = None)]
pub struct BashersApp {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Update Python dependencies
    Update {
        /// Package name to update (with fuzzy matching)
        package: Option<String>,
        /// Print commands without executing
        #[arg(long)]
        dry_run: bool,
    },
    /// Install project dependencies
    Setup {
        /// Use frozen/locked install
        #[arg(long)]
        frozen: bool,
        /// Remove .venv before install (implies --no-cache)
        #[arg(long)]
        rm: bool,
        /// Print commands without executing
        #[arg(long)]
        dry_run: bool,
    },
    /// List installed packages
    Show {
        /// Filter patterns
        patterns: Vec<String>,
    },
    /// Git home: checkout default branch, pull, fetch all
    Gh {
        /// Print commands without executing
        #[arg(long)]
        dry_run: bool,
    },
    /// Print version
    Version,
}
