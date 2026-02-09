use anyhow::Result;
use bashers::cli::BashersApp;
use clap::Parser;

fn main() -> Result<()> {
    let app = BashersApp::parse();

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
