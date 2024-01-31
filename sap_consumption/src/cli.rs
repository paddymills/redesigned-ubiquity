
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, InfoLevel};
use std::path::PathBuf;
use sysinteg_core::config::TomlConfig;

use crate::config::{CONFIG_FILE, SapConsumptionConfig};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
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
        let log_app_name = || -> anyhow::Result<String> {
            let cfg = SapConsumptionConfig::load(&PathBuf::from(CONFIG_FILE))?;
            
            Ok(cfg.logging_name)
        };

        if let Some(command) = &self.command {
            match command {
                Command::Install   => eventlog::register(&log_app_name()?)?,
                Command::Uninstall => eventlog::deregister(&log_app_name()?)?,
                Command::GenerateConfig => SapConsumptionConfig::generate(&PathBuf::from(CONFIG_FILE))?,
            };

            // false -> do not run executable
            Ok(false)
        } else {
            // true -> run executable
            Ok(true)
        }

    }

    pub fn log_level_filter(&self) -> log::LevelFilter {
        self.verbose.log_level_filter()
    }
}
