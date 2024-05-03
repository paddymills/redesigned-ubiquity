
use sndb_utils::{DisplayUpdate, ProgramInputHandler, Query, QueryTableUi};
use sndb_utils::{HEADER, Program};

use sysinteg_core::config::TomlConfig;
use sysinteg_db::DbConnParams;

use std::env;
use std::sync::mpsc;

const INSTRUCTIONS: &str = r#"
    ##############################################################
    #                     Sigmanest Database                     #
    #                     ------------------                     #
    #                  press ? for instructions                  #
    ##############################################################
"#;

// TODO: new table for each result
// TODO: results pagination
// TODO: maybe `ratatui` is better for this (https://ratatui.rs/)

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        let level = simplelog::LevelFilter::Info;
        let config = simplelog::ConfigBuilder::new()
            .add_filter_ignore_str("tiberius")
            .build();

        let _ = simplelog::WriteLogger::init(level, config, std::fs::File::create("updatedprograms.log").unwrap());
    }
    
    let cfg = match (env::var("SndbServer"), env::var("SndbDatabase")) {
        (Ok(server), Ok(database)) => DbConnParams { server, database },
        _ => match DbConnParams::load("db.toml") {
            Ok(config) => config,
            Err(error) => {
                eprintln!("Failed to parse `db.toml`");

                // wait for input to keep console window open
                println!("Press any key to exit...");
                let _ = std::io::stdin().read_line(&mut String::new());

                return Err(error)
            }
        }
    };

    let mut client = match cfg.connect().await {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Failed to connect to database: {}", error);

            // wait for input to keep console window open
            println!("Press any key to exit...");
            let _ = std::io::stdin().read_line(&mut String::new());

            return Err(error.into())
        }
    };

    let (tx_db, rx_db) = mpsc::channel::<String>();
    let (tx_display, rx_display) = mpsc::channel();

    let respond_to = tx_display.clone();
    tokio::spawn(async move {
        while let Ok(value) = rx_db.recv() {
            log::trace!("Query results frequested for `{}`", value);
            let message = match Query::try_from(value.as_str()) {
                Ok(query) => {
                    match query.execute(&mut client).await {
                        Ok(rows) => {
                            // send results to display thread
                            match rows.into_row().await {
                                Ok(Some(row)) => DisplayUpdate::DbResult(Program::from(&row)),
                                Ok(None) => DisplayUpdate::Message(format!("Program `{}` not found", value)),
                                Err(_) => DisplayUpdate::Message(String::from("Failed to get database results row"))
                            }
                        },
                        Err(_) => DisplayUpdate::Message(String::from("Failed to get database result"))
                    }
                },
                Err(e) => DisplayUpdate::Message(e.to_string())
            };

            let _ = respond_to.send(message);
        }

        log::trace!("Database thread shutting down");
    });

    QueryTableUi::new(&HEADER)
        .with_instructions(INSTRUCTIONS)
        .run_loop(&mut ProgramInputHandler::new(tx_db, tx_display), rx_display)
}