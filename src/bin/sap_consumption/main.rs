
// hide terminal window, if not a debug build
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod config;
mod dataset;

use chrono::{Local, NaiveDateTime, NaiveTime, Timelike};
use clap::Parser;
use eventlog::EventLog;
use log::Log;
use std::path::PathBuf;

use config::{CONFIG_FILE, SapConsumptionConfig};
use sysinteg::config::TomlConfig;
use dataset::Dataset;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    
    if args.handle_install()? {
        // load config
        let config = SapConsumptionConfig::load(&PathBuf::from(CONFIG_FILE))?;

        // init logger
        // we sill use the the most verbose logging level because fern will handle level filtering
        let event_logger: Box<dyn Log> = Box::new( EventLog::new(&config.logging_name, log::Level::max())? );
        fern::Dispatch::new()
            .level(log::LevelFilter::Warn)
            .level_for("sysint", args.log_level_filter())
            .level_for("sap_consumption", args.log_level_filter())
            .chain(event_logger)
            .apply()?;
        
        // pull data
        pull_interval(config).await?;
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
