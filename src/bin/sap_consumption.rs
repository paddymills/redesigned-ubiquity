
use chrono::{Duration, Local, NaiveDateTime, NaiveTime, Timelike};
use std::fs::File;
use std::io::Write;

use sysinteg::config::DbConnParams;
use sysinteg::db;

// TODO: store last queried time in database
const INTERVAL: i64 = 1;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let now = Local::now();
    let end = NaiveDateTime::new(now.date_naive(), NaiveTime::from_hms_opt(now.hour(), 0, 0).unwrap());
    let start = end - Duration::hours(INTERVAL);

    println!("pulling data for duration [{}, {})", start.format("%d/%m/%Y %H:%M"), end.format("%d/%m/%Y %H:%M"));

    let mut client = DbConnParams::load()?
        .connect()
            .await?;
    
    let prod = client
        .query("EXEC SapProduction @Start=@P1, @End=@P2", &[&start, &end]).await?
        .into_first_result().await?;

    let issue = client
        .query("EXEC SapIssueData @Start=@P1, @End=@P2", &[&start, &end]).await?
        .into_first_result().await?;

    write_data(prod, "Production", end);
    write_data(issue, "Issue", end);

    Ok(())
}

fn write_data(dataset: Vec<tiberius::Row>, name: &str, timestamp: NaiveDateTime) {
    // TODO: store on server
    let filename = format!("{}_{}.ready", name, timestamp.format("%Y%m%d%H%M%S"));
    let mut file = File::create(&filename)
        .expect(&format!("Failed to create file {}", &filename));

    if dataset.len() == 0 {
        println!("Dataset {} is empty", name);
    } else {
        println!("Writing dataset {}", name);
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
