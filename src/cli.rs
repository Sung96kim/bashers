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
        /// Run command in non-interactive mode - will auto select the closest matching library
        #[arg(short = 'y')]
        auto_select: bool,
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
    /// Kubernetes helper commands
    Kube {
        #[command(subcommand)]
        command: KubeCommands,
    },
    /// Print version
    Version,
    /// Self-management commands
    #[command(name = "self")]
    SelfCmd {
        #[command(subcommand)]
        command: SelfCommands,
    },
}

pub const TOPLEVEL_ALIAS_PARENTS: &[&str] = &["kube"];

#[derive(Subcommand)]
pub enum KubeCommands {
    /// Describe pod(s) and show Image lines (pod name regex-matched)
    Kmg {
        /// Pod name pattern (regex)
        pattern: String,
    },
    /// Follow logs from pods matching patterns (persists through restarts)
    Track {
        /// Pod name patterns (regex)
        patterns: Vec<String>,
        /// Only show WARNING/ERROR/CRITICAL log lines and tracebacks
        #[arg(long)]
        err_only: bool,
        /// Use simple output mode with context-switch headers instead of TUI
        #[arg(long)]
        simple: bool,
    },
}

#[derive(Subcommand)]
pub enum SelfCommands {
    /// Update bashers to the latest version
    Update,
}
