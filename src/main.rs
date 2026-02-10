use anyhow::Result;
use bashers::cli::{BashersApp, TOPLEVEL_ALIAS_PARENTS};
use clap::{CommandFactory, Parser};

fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().collect();
    if let Some(name) = args.get(1).map(String::as_str) {
        let root = BashersApp::command();
        let is_root_subcommand = root.get_subcommands().any(|c| c.get_name() == name);
        if !is_root_subcommand {
            if let Some(parent) = root.get_subcommands().find(|parent| {
                TOPLEVEL_ALIAS_PARENTS.contains(&parent.get_name())
                    && parent.get_subcommands().any(|c| c.get_name() == name)
            }) {
                args.insert(1, parent.get_name().to_string());
            }
        }
    }
    let app = BashersApp::parse_from(args);

    match app.command {
        Some(bashers::cli::Commands::Update {
            package,
            dry_run,
            auto_select,
        }) => {
            bashers::commands::update::run(package.as_deref(), dry_run, auto_select)?;
        }
        Some(bashers::cli::Commands::Setup {
            frozen,
            rm,
            dry_run,
        }) => {
            bashers::commands::setup::run(frozen, rm, dry_run)?;
        }
        Some(bashers::cli::Commands::Show { patterns }) => {
            bashers::commands::show::run(&patterns)?;
        }
        Some(bashers::cli::Commands::Gh { dry_run }) => {
            bashers::commands::gh::run(dry_run)?;
        }
        Some(bashers::cli::Commands::Kube { command }) => match command {
            bashers::cli::KubeCommands::Kmg { pattern } => {
                bashers::commands::kube::kmg::run(&pattern)?;
            }
            bashers::cli::KubeCommands::Track {
                patterns,
                err_only,
                simple,
            } => {
                bashers::commands::kube::track::run(&patterns, err_only, simple)?;
            }
        },
        Some(bashers::cli::Commands::Version) => {
            println!("bashers {}", env!("CARGO_PKG_VERSION"));
        }
        Some(bashers::cli::Commands::SelfCmd { command }) => match command {
            bashers::cli::SelfCommands::Update => {
                bashers::commands::self_cmd::update::run()?;
            }
        },
        None => {
            bashers::commands::help::run()?;
        }
    }

    Ok(())
}
