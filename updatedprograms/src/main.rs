
use updated_programs::{DisplayUpdate, ProgramInputHandler, QueryTableUi};
use updated_programs::{HEADER, Program};

use sysinteg_core::config::TomlConfig;
use sysinteg_db::DbConnParams;

use std::env;
use std::sync::mpsc;

const INSTRUCTIONS: &str = r#"
    ##############################################################
    #                      Updated Programs                      #
    #                     ------------------                     #
    #                  press ? for instructions                  #
    ##############################################################
"#;

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

    let (tx_db, rx_db) = mpsc::channel();
    let (tx_display, rx_display) = mpsc::channel();

    let respond_to = tx_display.clone();
    tokio::spawn(async move {
        while let Ok(program) = rx_db.recv() {
            log::trace!("Program results frequested for `{}`", program);
            let query_result = client.query("EXEC GetProgramStatus @ProgramName=@P1", &[&program]).await;

            let message = match query_result {
                Ok(rows) => {
                    // send results to display thread
                    match rows.into_row().await {
                        Ok(Some(row)) => DisplayUpdate::DbResult(Program::from(&row)),
                        Ok(None) => DisplayUpdate::Message(format!("Program `{}` not found", program)),
                        Err(_) => DisplayUpdate::Message(String::from("Failed to get database results row"))
                    }
                },
                Err(_) => DisplayUpdate::Message(String::from("Failed to get database result"))
            };

            let _ = respond_to.send(message);
        }

        log::trace!("Database thread shutting down");
    });

    QueryTableUi::new(&HEADER)
        .with_instructions(INSTRUCTIONS)
        .run_loop(&mut ProgramInputHandler::new(tx_db, tx_display), rx_display)
}