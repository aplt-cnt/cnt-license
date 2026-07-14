use anyhow::{anyhow, Result};
#[cfg(target_os = "linux")]
use colored::Colorize;

use crate::cli::ServiceAction;

const SERVICE_NAME: &str = "clicense-server";
#[cfg(target_os = "linux")]
const UNIT_FILE: &str = "/etc/systemd/system/clicense-server.service";
const DEFAULT_SERVICE_CONFIG_DIR: &str = "/etc/clicense-server";

/// Executes the `service` subcommand: dispatches systemd lifecycle or data operations.
pub fn execute(config_dir: Option<&str>, action: ServiceAction, verbose: bool) -> Result<()> {
    match action {
        ServiceAction::Install { config_dir: svc_config_dir, no_enable, force } => {
            service_install(svc_config_dir.as_deref(), no_enable, force, verbose)
        }
        ServiceAction::Uninstall => service_uninstall(verbose),
        ServiceAction::Start => run_systemctl(&["start", SERVICE_NAME], verbose),
        ServiceAction::Stop => run_systemctl(&["stop", SERVICE_NAME], verbose),
        ServiceAction::Restart => run_systemctl(&["restart", SERVICE_NAME], verbose),
        ServiceAction::Enable => run_systemctl(&["enable", SERVICE_NAME], verbose),
        ServiceAction::Disable => run_systemctl(&["disable", SERVICE_NAME], verbose),
        ServiceAction::Status => run_systemctl(&["status", SERVICE_NAME], verbose),
        ServiceAction::Reload => run_systemctl(&["daemon-reload"], verbose),
        ServiceAction::Add { file, name, force, display_name, description, spdx_id, permissions, conditions, limitations, keywords, custom } => {
            let svc_cfg = config_dir.or(Some(DEFAULT_SERVICE_CONFIG_DIR));
            crate::command::add::execute(
                svc_cfg,
                &file, &name, force,
                None, None,
                display_name.as_deref(), description.as_deref(), spdx_id.as_deref(),
                &permissions, &conditions, &limitations, &keywords, &custom,
                verbose,
            )
        }
        ServiceAction::Remove { names, all } => {
            let svc_cfg = config_dir.or(Some(DEFAULT_SERVICE_CONFIG_DIR));
            crate::command::remove::execute(svc_cfg, &names, all, None, None, verbose)
        }
        ServiceAction::List { name } => {
            let svc_cfg = config_dir.or(Some(DEFAULT_SERVICE_CONFIG_DIR));
            crate::command::list::execute(svc_cfg, name.as_deref(), None, None, verbose)
        }
        ServiceAction::Clone { url, force } => {
            let svc_cfg = config_dir.or(Some(DEFAULT_SERVICE_CONFIG_DIR));
            crate::command::clone::execute(svc_cfg, &url, None, None, force, verbose)
        }
        ServiceAction::Source { name } => {
            let svc_cfg = config_dir.or(Some(DEFAULT_SERVICE_CONFIG_DIR));
            crate::command::source::execute(svc_cfg, &name, None, verbose)
        }
        ServiceAction::Export { output } => {
            let svc_cfg = config_dir.or(Some(DEFAULT_SERVICE_CONFIG_DIR));
            crate::command::export::execute(svc_cfg, output.as_deref(), None, None, verbose)
        }
        ServiceAction::Import { file, force } => {
            let svc_cfg = config_dir.or(Some(DEFAULT_SERVICE_CONFIG_DIR));
            crate::command::import::execute(svc_cfg, &file, force, None, None, verbose)
        }
    }
}

fn service_install(
    svc_config_dir: Option<&str>,
    no_enable: bool,
    force: bool,
    verbose: bool,
) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (svc_config_dir, no_enable, force, verbose);
        return Err(anyhow!("Service management is only supported on Linux."));
    }

    #[cfg(target_os = "linux")]
    {
        let unit_path = std::path::Path::new(UNIT_FILE);
        if unit_path.exists() && !force {
            return Err(anyhow!(
                "Service unit already exists at '{}'. Use {} to overwrite.",
                UNIT_FILE,
                "clicense-server service install --force".yellow()
            ));
        }

        let config_dir = svc_config_dir.unwrap_or(DEFAULT_SERVICE_CONFIG_DIR);
        let binary_path = std::env::current_exe()
            .map_err(|e| anyhow!("Failed to determine binary path: {}", e))?;

        let unit_content = format!(
            r#"[Unit]
Description=clicense-server - Open Source License API Server
After=network.target
Documentation=https://clicense.top

[Service]
Type=simple
ExecStart={} run --config-dir {}
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=clicense-server

[Install]
WantedBy=multi-user.target
"#,
            binary_path.display(),
            config_dir
        );

        if verbose {
            println!("{} Binary: {}", "·".dimmed(), binary_path.display().to_string().cyan());
            println!("{} Config dir: {}", "·".dimmed(), config_dir.cyan());
            println!("{} Unit file: {}", "·".dimmed(), UNIT_FILE.cyan());
            println!();
            println!("{}", unit_content.trim());
            println!();
        }

        std::fs::write(unit_path, &unit_content)
            .map_err(|e| anyhow!("Failed to write service unit '{}': {}", UNIT_FILE, e))?;

        println!(
            "{} Service unit written to {}",
            "✓".green().bold(),
            UNIT_FILE.cyan()
        );

        run_systemctl(&["daemon-reload"], verbose)?;

        if !no_enable {
            run_systemctl(&["enable", SERVICE_NAME], verbose)?;
        }

        println!();
        println!(
            "{} Run {} to start the service.",
            "💡".yellow(),
            "clicense-server service start".cyan()
        );

        Ok(())
    }
}

fn service_uninstall(verbose: bool) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        return Err(anyhow!("Service management is only supported on Linux."));
    }

    #[cfg(target_os = "linux")]
    {
        let unit_path = std::path::Path::new(UNIT_FILE);
        if !unit_path.exists() {
            println!(
                "  {} Service unit not found at '{}'",
                "—".dimmed(),
                UNIT_FILE
            );
            return Ok(());
        }

        let _ = run_systemctl(&["stop", SERVICE_NAME], verbose);
        let _ = run_systemctl(&["disable", SERVICE_NAME], verbose);

        std::fs::remove_file(unit_path)
            .map_err(|e| anyhow!("Failed to remove '{}': {}", UNIT_FILE, e))?;

        run_systemctl(&["daemon-reload"], verbose)?;

        println!(
            "{} Service uninstalled successfully.",
            "✓".green().bold()
        );

        Ok(())
    }
}

fn run_systemctl(args: &[&str], verbose: bool) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (args, verbose);
        return Err(anyhow!("systemctl is only available on Linux."));
    }

    #[cfg(target_os = "linux")]
    {
        if verbose {
            println!("{} systemctl {}", "·".dimmed(), args.join(" ").cyan());
        }

        let status = std::process::Command::new("systemctl")
            .args(args)
            .status()
            .map_err(|e| anyhow!("Failed to run systemctl: {}", e))?;

        if !status.success() {
            return Err(anyhow!(
                "systemctl {} exited with status {}",
                args.join(" "),
                status
            ));
        }

        Ok(())
    }
}
