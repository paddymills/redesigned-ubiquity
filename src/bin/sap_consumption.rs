
use chrono::{Duration, Local, NaiveDateTime, NaiveTime, Timelike};
use sysinteg::config::DbConnParams;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let now = Local::now();
    let end = NaiveDateTime::new(now.date_naive(), NaiveTime::from_hms_opt(now.hour(), 0, 0).unwrap());
    let start = end - Duration::hours(1);

    println!("pulling data for duration [{}, {})", start.format("%d/%m/%Y %H:%M"), end.format("%d/%m/%Y %H:%M"));

    let mut client = DbConnParams::load()?.connect().await?;
    client
        .query(
            "EXECUTE SapProductionData @Start=@P1, @End=@P2",
            &[&start, &end]
        )
            .await?
        .into_first_result()
            .await?
        .iter()
        .for_each(|row| println!("{:?}", row));

    Ok(())
}
