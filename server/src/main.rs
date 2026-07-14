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
    let config_dir = cli.config_dir.as_deref();

    match cli.command {
        cli::Commands::Init { licenses_dir, meta_dir, force } => {
            command::init::execute(config_dir, licenses_dir.as_deref(), meta_dir.as_deref(), force, verbose)?;
        }
        cli::Commands::Config { key, value, list, reset } => {
            command::config_cmd::execute(
                config_dir,
                key.as_deref(),
                value.as_deref(),
                list,
                reset.as_deref(),
                verbose,
            )?;
        }
        cli::Commands::Clone { url, licenses_dir, meta_dir, force } => {
            command::clone::execute(config_dir, &url, licenses_dir.as_deref(), meta_dir.as_deref(), force, verbose)?;
        }
        cli::Commands::Version => {
            println!("clicense-server {}", env!("CARGO_PKG_VERSION"));
        }
        cli::Commands::Run { host, port, licenses_dir, meta_dir } => {
            command::run::execute(config_dir, host.as_deref(), port, licenses_dir.as_deref(), meta_dir.as_deref(), verbose)?;
        }
        cli::Commands::Add { file, name, force, licenses_dir, meta_dir, display_name, description, spdx_id, permissions, conditions, limitations, keywords, custom } => {
            command::add::execute(
                config_dir,
                &file, &name, force,
                licenses_dir.as_deref(), meta_dir.as_deref(),
                display_name.as_deref(), description.as_deref(), spdx_id.as_deref(),
                &permissions, &conditions, &limitations, &keywords, &custom,
                verbose,
            )?;
        }
        cli::Commands::Remove { names, all, licenses_dir, meta_dir } => {
            command::remove::execute(config_dir, &names, all, licenses_dir.as_deref(), meta_dir.as_deref(), verbose)?;
        }
        cli::Commands::List { name, licenses_dir, meta_dir } => {
            command::list::execute(config_dir, name.as_deref(), licenses_dir.as_deref(), meta_dir.as_deref(), verbose)?;
        }
        cli::Commands::Source { name, licenses_dir } => {
            command::source::execute(config_dir, &name, licenses_dir.as_deref(), verbose)?;
        }
        cli::Commands::Export { output, licenses_dir, meta_dir } => {
            command::export::execute(config_dir, output.as_deref(), licenses_dir.as_deref(), meta_dir.as_deref(), verbose)?;
        }
        cli::Commands::Import { file, force, licenses_dir, meta_dir } => {
            command::import::execute(config_dir, &file, force, licenses_dir.as_deref(), meta_dir.as_deref(), verbose)?;
        }
        cli::Commands::Service { action } => {
            command::service::execute(config_dir, action, verbose)?;
        }
    }
    Ok(())
}
