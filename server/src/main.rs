mod cli;
mod command;
mod config;
mod data;
mod handlers;
mod http;
mod models;
mod state;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e.to_string().red());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = cli::Cli::parse();
    let verbose = cli.verbose;

    match cli.command {
        cli::Commands::Init { licenses_dir, force } => {
            command::init::execute(licenses_dir.as_deref(), force, verbose)?;
        }
        cli::Commands::Config {
            key,
            value,
            list,
            reset,
        } => {
            command::config_cmd::execute(
                key.as_deref(),
                value.as_deref(),
                list,
                reset.as_deref(),
                verbose,
            )?;
        }
        cli::Commands::Clone {
            url,
            licenses_dir,
            force,
        } => {
            command::clone::execute(&url, licenses_dir.as_deref(), force, verbose)?;
        }
        cli::Commands::Version => {
            println!("clicense-server {}", env!("CARGO_PKG_VERSION"));
        }
        cli::Commands::Run {
            host,
            port,
            licenses_dir,
        } => {
            command::run::execute(host.as_deref(), port, licenses_dir.as_deref(), verbose)?;
        }
        cli::Commands::Add {
            file,
            name,
            force,
            licenses_dir,
        } => {
            command::add::execute(&file, &name, force, licenses_dir.as_deref(), verbose)?;
        }
        cli::Commands::Remove {
            names,
            all,
            licenses_dir,
        } => {
            command::remove::execute(&names, all, licenses_dir.as_deref(), verbose)?;
        }
        cli::Commands::List { name, licenses_dir } => {
            command::list::execute(name.as_deref(), licenses_dir.as_deref(), verbose)?;
        }
    }
    Ok(())
}
