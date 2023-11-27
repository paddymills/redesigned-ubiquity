

use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use std::path::PathBuf;
use sysinteg::config::Config;

use crate::config::{CONFIG_FILE, SapConsumptionConfig};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    
    #[command(flatten)]
    pub verbose: Verbosity,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Install (register to the Windows Event Log)
    Install,
    /// uninstall (deregister from the Windows Event Log)
    Uninstall,
    /// generate example config
    GenerateConfig,
}

impl Cli {
    pub fn handle_install(&self) -> anyhow::Result<bool> {
        if let Some(command) = &self.command {
            match command {
                Command::Install => eventlog::register("Sap Consumption")?,
                Command::Uninstall => eventlog::deregister("Sap Consumption")?,
                Command::GenerateConfig => SapConsumptionConfig::generate(&PathBuf::from(CONFIG_FILE))?,
            };

            // false -> do not run executable
            Ok(false)
        } else {
            // true -> run executable
            Ok(true)
        }

    }
}
