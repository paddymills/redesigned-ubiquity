
// hide terminal window, if not a debug build and terminal feature is not enabled
#![cfg_attr(all(not(debug_assertions), not(feature = "terminal")), windows_subsystem = "windows")]

mod cli;
mod config;
mod dataset;
mod logging;

use chrono::{Local, NaiveDateTime, NaiveTime, Timelike};
use clap::Parser;
use std::path::PathBuf;

use config::{CONFIG_FILE, SapConsumptionConfig};
use dataset::Dataset;
use sysinteg_core::config::TomlConfig;
use logging::EventAndDbLogger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    
    if args.handle_install()? {
        // load config
        let config = SapConsumptionConfig::load(&PathBuf::from(CONFIG_FILE))?;

        // init logging
        let logger = EventAndDbLogger::init(&config.logging_name, &config.database, args.log_level_filter(), &[module_path!()]).await?;
        
        // pull data
        pull_interval(config).await?;

        // clean up logger
        logger.finalize().await;
    }

    Ok(())
}

async fn pull_interval(config: SapConsumptionConfig) -> anyhow::Result<()> {
    let now = Local::now();
    let end = NaiveDateTime::new(now.date_naive(), NaiveTime::from_hms_opt(now.hour(), 0, 0).unwrap());

    log::info!("pulling data from last run until {}", end.format("%d/%m/%Y %H:%M"));

    let mut client = config.database.connect().await?;
    Dataset::Production.pull_data(&mut client, end, &config.output_dir).await?;
    Dataset::Issue.pull_data(&mut client, end, &config.output_dir).await?;

    Ok(())
}
