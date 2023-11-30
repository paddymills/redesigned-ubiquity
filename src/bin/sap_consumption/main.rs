
// hide terminal window, if not a debug build and terminal feature is not enabled
#![cfg_attr(all(not(debug_assertions), not(feature = "terminal")), windows_subsystem = "windows")]

mod cli;
mod config;
mod dataset;

use chrono::{Local, NaiveDateTime, NaiveTime, Timelike};
use clap::Parser;
use eventlog::EventLog;
use log::Log;
use std::path::PathBuf;

use config::{CONFIG_FILE, SapConsumptionConfig};
use dataset::Dataset;
use sysinteg::config::TomlConfig;
use sysinteg::logging::MssqlDbLogger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    
    if args.handle_install()? {
        // load config
        let config = SapConsumptionConfig::load(&PathBuf::from(CONFIG_FILE))?;

        init_logging(&config, &args).await?;
        
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

async fn init_logging(config: &SapConsumptionConfig, args: &cli::Cli) -> anyhow::Result<()> {
    // we sill use the the most verbose logging level because fern will handle level filtering
    let loggers_cfg_level = log::Level::max();

    let event_logger: Box<dyn Log> = Box::new(
        EventLog::new(&config.logging_name, loggers_cfg_level)? );
    let db_logger: Box<dyn Log> = Box::new(
        MssqlDbLogger::new(&config.logging_name, config.database.connect().await?, loggers_cfg_level) );


    let logger = fern::Dispatch::new()
        // Tokio/Tiberius (and maybe other dependencies) do some logging of their own.
        //  This allows us to use a lower log level, while silencing their verbose logs.
        .level(log::LevelFilter::Warn)
        .level_for("sysint", args.log_level_filter())
        .level_for("sap_consumption", args.log_level_filter())

        // log to Windows Event Log
        .chain(event_logger)

        // database logger (Actor pattern where database transactions happen on a separate thread)
        .chain(db_logger);

    // log to stdout if not a debug build or terminal feature is enabled
    #[cfg(all(not(debug_assertions), not(feature = "terminal")))]
    logger.chain(std::io::stdout());

    // register as the global logger (this will fail if a global logger is already set)
    logger.apply()?;

    Ok(())
}
