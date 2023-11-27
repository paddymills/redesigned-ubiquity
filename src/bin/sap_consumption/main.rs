
// hide terminal window, if not a debug build
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod config;

use chrono::{Duration, Local, NaiveDateTime, NaiveTime, Timelike};
use config::{CONFIG_FILE, SapConsumptionConfig};

use std::fs::File;
use std::path::PathBuf;
use std::io::Write;

use clap::Parser;

use sysinteg::{config::TomlConfig, db};

// TODO: store last queried time in database
const INTERVAL: i64 = 1;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    
    if args.handle_install()? {
        // load config
        let config = SapConsumptionConfig::load(&PathBuf::from(CONFIG_FILE))?;

        // init logger
        eventlog::init(&config.logging_name, args.log_level())?;
        
        pull_interval(config).await?;
    }

    Ok(())
}

async fn pull_interval(config: SapConsumptionConfig) -> anyhow::Result<()> {
    let now = Local::now();
    let end = NaiveDateTime::new(now.date_naive(), NaiveTime::from_hms_opt(now.hour(), 0, 0).unwrap());
    let start = end - Duration::hours(INTERVAL);

    log::info!("pulling data for duration [{}, {})", start.format("%d/%m/%Y %H:%M"), end.format("%d/%m/%Y %H:%M"));

    let mut client = config.database.connect().await?;
    
    log::trace!("pulling Production dataset");
    let prod = client
        .query("EXEC SapProductionData @Start=@P1, @End=@P2", &[&start, &end]).await?
        .into_first_result().await?;

        
    log::trace!("pulling Issue dataset");
    let issue = client
        .query("EXEC SapIssueData @Start=@P1, @End=@P2", &[&start, &end]).await?
        .into_first_result().await?;

    write_data(prod,  "Production", end, &config.output_dir);
    write_data(issue, "Issue",      end, &config.output_dir);

    Ok(())
}

fn write_data(dataset: Vec<tiberius::Row>, name: &str, timestamp: NaiveDateTime, outdir: &PathBuf) {
    if dataset.len() == 0 {
        log::debug!("Dataset {} is empty", name);
    } else {
        // TODO: store on server
        let filename = format!("{}_{}.ready", name, timestamp.format("%Y%m%d%H%M%S"));
        let filename = outdir.join(filename);
        
        let mut file = File::create(&filename)
            .map_err(|_| log::error!("Failed to create file {}", &filename.to_str().unwrap()))
            .unwrap();
    
        log::info!("Writing dataset {}", name);
        dataset
            .into_iter()
            // convert row to tab delimited string
            .map(|row| db::row_to_string(row))
            .for_each(|row| {
                file.write_all(row.as_bytes())
                    .expect("Failed to write data to file");
                file.write_all(b"\n")
                    .expect("failed to write newline character");
                });
    }
}
