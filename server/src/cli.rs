use clap::{Parser, Subcommand};

/// clicense-server - Open source license API server
#[derive(Parser, Debug)]
#[command(name = "clicense-server", version, about = "An open source license API server", arg_required_else_help = true)]
pub struct Cli {
    /// Enable verbose output (show config paths, file details, HTTP info, etc.)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Override the config directory (default: /etc/clicense-server or ~/.clicense-server)
    #[arg(long, global = true)]
    pub config_dir: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the server (write built-in license templates)
    Init {
        #[arg(long)]
        licenses_dir: Option<String>,
        #[arg(long)]
        meta_dir: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// View or set server configuration
    Config {
        key: Option<String>,
        value: Option<String>,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        reset: Option<String>,
    },
    /// Clone license templates from a remote API server
    Clone {
        url: String,
        #[arg(long)]
        licenses_dir: Option<String>,
        #[arg(long)]
        meta_dir: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// Show version number
    Version,
    /// Start the API server
    Run {
        #[arg(long)]
        host: Option<String>,
        #[arg(long)]
        port: Option<u16>,
        #[arg(long)]
        licenses_dir: Option<String>,
        #[arg(long)]
        meta_dir: Option<String>,
    },
    /// Add a license template
    Add {
        file: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        licenses_dir: Option<String>,
        #[arg(long)]
        meta_dir: Option<String>,
        #[arg(long)]
        display_name: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        spdx_id: Option<String>,
        #[arg(long)]
        permissions: Vec<String>,
        #[arg(long)]
        conditions: Vec<String>,
        #[arg(long)]
        limitations: Vec<String>,
        #[arg(long)]
        keywords: Vec<String>,
        #[arg(long, value_parser = parse_key_value)]
        custom: Vec<String>,
    },
    /// Remove license templates
    Remove {
        names: Vec<String>,
        #[arg(long, conflicts_with = "names")]
        all: bool,
        #[arg(long)]
        licenses_dir: Option<String>,
        #[arg(long)]
        meta_dir: Option<String>,
    },
    /// List license templates
    List {
        name: Option<String>,
        #[arg(long)]
        licenses_dir: Option<String>,
        #[arg(long)]
        meta_dir: Option<String>,
    },
    /// Output raw license template content
    Source {
        name: String,
        #[arg(long)]
        licenses_dir: Option<String>,
    },
    /// Export all licenses to a .zip file
    Export {
        #[arg(long)]
        output: Option<String>,
        #[arg(long)]
        licenses_dir: Option<String>,
        #[arg(long)]
        meta_dir: Option<String>,
    },
    /// Import licenses from a .zip or .toml file
    Import {
        file: String,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        licenses_dir: Option<String>,
        #[arg(long)]
        meta_dir: Option<String>,
    },
    /// Manage the systemd service or license data
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ServiceAction {
    /// Install systemd service unit (requires root)
    Install {
        #[arg(long)]
        config_dir: Option<String>,
        #[arg(long)]
        no_enable: bool,
        #[arg(long)]
        force: bool,
    },
    /// Uninstall systemd service unit (requires root)
    Uninstall,
    /// Start the service via systemctl
    Start,
    /// Stop the service via systemctl
    Stop,
    /// Restart the service via systemctl
    Restart,
    /// Enable auto-start on boot via systemctl
    Enable,
    /// Disable auto-start on boot via systemctl
    Disable,
    /// Show service status via systemctl
    Status,
    /// Reload systemd daemon (systemctl daemon-reload)
    Reload,
    /// Add a license to the service
    Add {
        file: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        display_name: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        spdx_id: Option<String>,
        #[arg(long)]
        permissions: Vec<String>,
        #[arg(long)]
        conditions: Vec<String>,
        #[arg(long)]
        limitations: Vec<String>,
        #[arg(long)]
        keywords: Vec<String>,
        #[arg(long, value_parser = parse_key_value)]
        custom: Vec<String>,
    },
    /// Remove licenses from the service
    Remove {
        names: Vec<String>,
        #[arg(long, conflicts_with = "names")]
        all: bool,
    },
    /// List licenses in the service
    List {
        name: Option<String>,
    },
    /// Clone licenses from a remote API server
    Clone {
        url: String,
        #[arg(long)]
        force: bool,
    },
    /// Output raw license template content
    Source {
        name: String,
    },
    /// Export all licenses from the service to a .zip file
    Export {
        #[arg(long)]
        output: Option<String>,
    },
    /// Import licenses into the service from a .zip or .toml file
    Import {
        file: String,
        #[arg(long)]
        force: bool,
    },
}

fn parse_key_value(s: &str) -> Result<String, String> {
    if s.contains('=') {
        Ok(s.to_string())
    } else {
        Err(format!(
            "Invalid key=value pair: '{}'. Expected format: KEY=VALUE",
            s
        ))
    }
}
