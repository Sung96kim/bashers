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
    /// Git helper commands
    Git {
        #[command(subcommand)]
        command: GitCommands,
    },
    /// Kubernetes helper commands
    Kube {
        #[command(subcommand)]
        command: KubeCommands,
    },
    /// Docker helper commands
    Docker {
        #[command(subcommand)]
        command: DockerCommands,
    },
    /// Print version
    Version,
    /// Run a command repeatedly and highlight output changes (use -- to separate options from command)
    Watch {
        /// Seconds between runs
        #[arg(short = 'n', long, default_value = "2")]
        interval: u64,
        /// Disable diff highlighting; show raw output only
        #[arg(long)]
        no_diff: bool,
        /// Command and arguments to run (e.g. watch -n 1 -- ls -la)
        #[arg(required = true, num_args = 1.., value_terminator = "--")]
        command: Vec<String>,
    },
    /// Self-management commands
    #[command(name = "self")]
    SelfCmd {
        #[command(subcommand)]
        command: SelfCommands,
    },
}

pub const TOPLEVEL_ALIAS_PARENTS: &[&str] = &["docker", "git", "kube"];

#[derive(Subcommand)]
pub enum GitCommands {
    /// Sync repo: checkout default branch, pull, fetch (use --current to sync current branch only)
    Sync {
        /// Sync current branch only (pull + fetch, no checkout)
        #[arg(long)]
        current: bool,
        /// Print commands without executing
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum DockerCommands {
    /// Build an image from a Dockerfile
    Build {
        /// Path to the Dockerfile (default: Dockerfile in current directory)
        #[arg(short = 'f', long, value_name = "PATH")]
        dockerfile: Option<std::path::PathBuf>,
        /// Image name and optional tag (e.g. myapp:latest)
        #[arg(short = 't', long)]
        tag: Option<String>,
        /// Do not use cache when building
        #[arg(long)]
        no_cache: bool,
        /// Build context path (default: directory of the Dockerfile)
        #[arg(short = 'c', long, value_name = "PATH")]
        context: Option<std::path::PathBuf>,
    },
}

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
