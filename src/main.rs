use anyhow::Result;
use bashers::cli::BashersApp;
use clap::Parser;

fn main() -> Result<()> {
    let app = BashersApp::parse();

    match app.command {
        Some(bashers::cli::Commands::Update { package, dry_run, auto_select }) => {
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
        Some(bashers::cli::Commands::Version) => {
            println!("bashers {}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            bashers::commands::help::run()?;
        }
    }

    Ok(())
}
