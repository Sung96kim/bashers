pub mod cli;
pub mod commands;
pub(crate) mod tui;
pub mod utils;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use crate::cli::{BashersApp, TOPLEVEL_ALIAS_PARENTS};

pub fn run(args: Vec<String>) -> Result<()> {
    let mut args = args;
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
        Some(cli::Commands::Update {
            packages,
            dry_run,
            auto_select,
            verbose,
        }) => commands::update::run(&packages, dry_run, auto_select, verbose)?,
        Some(cli::Commands::Setup {
            frozen,
            rm,
            dry_run,
        }) => commands::setup::run(frozen, rm, dry_run)?,
        Some(cli::Commands::Show { patterns }) => commands::show::run(&patterns)?,
        Some(cli::Commands::Git { command }) => match command {
            cli::GitCommands::Sync { current, dry_run } => {
                commands::git::sync::run(current, dry_run)?
            }
        },
        Some(cli::Commands::Kube { command }) => match command {
            cli::KubeCommands::Kmg { patterns } => commands::kube::kmg::run(&patterns)?,
            cli::KubeCommands::Track {
                patterns,
                err_only,
                simple,
            } => commands::kube::track::run(&patterns, err_only, simple)?,
        },
        Some(cli::Commands::Docker { command }) => match command {
            cli::DockerCommands::Build {
                dockerfile,
                tag,
                no_cache,
                context,
            } => commands::docker::build::run(
                dockerfile.as_deref(),
                tag.as_deref(),
                no_cache,
                context.as_deref(),
            )?,
        },
        Some(cli::Commands::Version) => println!("v{}", env!("CARGO_PKG_VERSION")),
        Some(cli::Commands::Watch {
            command,
            interval,
            no_diff,
        }) => commands::watch::run(&command, interval, no_diff)?,
        Some(cli::Commands::SelfCmd { command }) => match command {
            cli::SelfCommands::Update => commands::self_cmd::update::run()?,
        },
        None => commands::help::run()?,
    }

    Ok(())
}

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pyfunction]
fn run_cli(py: Python<'_>) -> PyResult<()> {
    let sys = py.import("sys")?;
    let argv: Vec<String> = sys.getattr("argv")?.extract::<Vec<String>>()?;
    let args = if argv.len() <= 1 {
        vec!["bashers".to_string()]
    } else {
        let mut a = vec!["bashers".to_string()];
        a.extend(argv[1..].to_vec());
        a
    };
    run(args).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

#[cfg(feature = "pyo3")]
#[pymodule]
fn bashers(m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_function(pyo3::wrap_pyfunction!(run_cli, m)?)?;
    Ok(())
}
