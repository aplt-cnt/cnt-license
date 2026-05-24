use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod command;
mod config;
mod http;
mod license;
mod metadata;

const VERSION: &str = "0.1.0";

/// clicense - A CLI tool for generating open source license files
#[derive(Parser, Debug)]
#[command(name = "clicense", version = VERSION, about = "A CLI tool for generating open source license files", long_about = None)]
struct Cli {
    /// Enable verbose output (show resolved config, file paths, HTTP details, etc.)
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show the version number
    Version,
    /// Generate a new license file
    New {
        /// License identifier (e.g., mit, apache-2.0, gpl-3.0, bsd-3-clause, bsd-2-clause, mpl-2.0, lgpl-3.0, unlicense, isc, epl-2.0)
        license_id: Option<String>,
        /// Output file name
        #[arg(short, long)]
        output: Option<String>,
        /// Copyright year
        #[arg(short, long)]
        year: Option<String>,
        /// Copyright holder name
        #[arg(short, long)]
        author: Option<String>,
    },
    /// View or set configuration values
    Config {
        /// Configuration key
        key: Option<String>,
        /// Configuration value (omit to view current value)
        value: Option<String>,
        /// List all configuration keys and their current values
        #[arg(long)]
        list: bool,
        /// Reset a configuration key to its default value
        #[arg(long)]
        reset: Option<String>,
    },
    /// Update license templates from remote URL
    Update {
        /// Override the update URL (defaults to configured update_url)
        #[arg(long)]
        update_url: Option<String>,
    },
    /// Add a custom license template
    Add {
        /// Path to the license template file
        file: String,
        /// Name/identifier for the custom license
        #[arg(long)]
        name: String,
        /// Overwrite if a custom license with this name already exists
        #[arg(long)]
        force: bool,
    },
    /// Remove one or more custom license templates
    Remove {
        /// Names/identifiers of the custom licenses to remove
        names: Vec<String>,
        /// Remove ALL custom licenses (use with caution)
        #[arg(long, conflicts_with = "names")]
        all: bool,
    },
    /// List installed licenses or view detailed info
    List {
        /// License name to show detailed info (omit to list all)
        license_name: Option<String>,
        /// Show only built-in licenses
        #[arg(long)]
        builtin: bool,
        /// Show only custom licenses
        #[arg(long, conflicts_with = "builtin")]
        custom: bool,
    },
    /// Interact with the remote license server
    Online {
        /// Override the server URL (defaults to configured update_url)
        #[arg(long)]
        online_url: Option<String>,

        #[command(subcommand)]
        command: OnlineCommands,
    },
    /// Output the raw content of a license
    Source {
        /// License name
        license_name: String,
        /// Copyright year (replaces {year} placeholder)
        #[arg(short, long)]
        year: Option<String>,
        /// Copyright holder (replaces {author} placeholder)
        #[arg(short, long)]
        author: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum OnlineCommands {
    /// List all licenses available on the remote server
    List,
    /// Show detailed info about a license from the remote server
    License {
        /// License name
        name: String,
    },
    /// Output raw license content from the remote server
    Source {
        /// License name
        name: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e.to_string().red());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let verbose = cli.verbose;

    match cli.command {
        Commands::Version => {
            println!("clicense {}", VERSION);
        }
        Commands::New {
            license_id,
            output,
            year,
            author,
        } => {
            command::new::execute(
                license_id.as_deref(),
                output.as_deref(),
                year.as_deref(),
                author.as_deref(),
                verbose,
            )?;
        }
        Commands::Config { key, value, list, reset } => {
            command::config_cmd::execute(
                key.as_deref(),
                value.as_deref(),
                list,
                reset.as_deref(),
                verbose,
            )?;
        }
        Commands::Update { update_url } => {
            command::update::execute(update_url.as_deref(), verbose)?;
        }
        Commands::Add { file, name, force } => {
            command::add::execute(&file, &name, force, verbose)?;
        }
        Commands::Remove { names, all } => {
            if all {
                command::remove::execute_all(verbose)?;
            } else {
                command::remove::execute(&names, verbose)?;
            }
        }
        Commands::List { license_name, builtin, custom } => {
            command::list::execute(license_name.as_deref(), builtin, custom, verbose)?;
        }
        Commands::Online { online_url, command } => match command {
            OnlineCommands::List => command::online::execute_list(online_url.as_deref(), verbose)?,
            OnlineCommands::License { name } => command::online::execute_license(&name, online_url.as_deref(), verbose)?,
            OnlineCommands::Source { name } => command::online::execute_source(&name, online_url.as_deref(), verbose)?,
        },
        Commands::Source {
            license_name,
            year,
            author,
        } => {
            command::source::execute(&license_name, year.as_deref(), author.as_deref(), verbose)?;
        }
    }

    Ok(())
}
