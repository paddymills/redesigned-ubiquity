
use updatedprograms::{DisplayUpdate, InputHandler, TableTerminal};
use updatedprograms::{HEADER, Program};

use sysinteg_core::config::TomlConfig;
use sysinteg_db::DbConnParams;

use std::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        let level = simplelog::LevelFilter::Info;
        let config = simplelog::ConfigBuilder::new()
            .add_filter_ignore_str("tiberius")
            .build();

        let _ = simplelog::WriteLogger::init(level, config, std::fs::File::create("updatedprograms.log").unwrap());
    }
    
    let cfg = DbConnParams::load("db.toml")?;
    let mut client = cfg.connect().await.expect("Failed to connect to database");

    let (tx_db, rx_db) = mpsc::channel();
    let (tx_display, rx_display) = mpsc::channel();

    tokio::spawn(async move {
        while let Ok(program) = rx_db.recv() {
            log::trace!("Program results frequested for `{}`", program);
            let query_result = client.query("EXEC GetProgramStatus @ProgramName=@P1", &[&program]).await;

            let message = match query_result {
                Ok(rows) => {
                    // send results to display thread
                    match rows.into_row().await {
                        Ok(Some(row)) => DisplayUpdate::DbResult(Program::from(&row)),
                        Ok(None) => DisplayUpdate::DbMessage(format!("Program `{}` not found", program)),
                        Err(_) => DisplayUpdate::DbMessage(String::from("Failed to get database results row"))
                    }
                },
                Err(_) => DisplayUpdate::DbMessage(String::from("Failed to get database result"))
            };

            let _ = tx_display.send(message);
        }

        log::trace!("Database thread shutting down");
    });

    let handler = InputHandler::new("Program", tx_db);
    TableTerminal::new(HEADER, handler).input_loop(rx_display)
}
