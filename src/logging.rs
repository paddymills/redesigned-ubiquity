
//! logging framework

use chrono::{Local, NaiveDateTime};
use log::Level;
use tokio::sync::mpsc::{self, Sender, Receiver};

use super::db::*;

/// Logger that logs to a MSSQL database
#[derive(Debug)]
pub struct MssqlDbLogger {
    app: String,
    tx: Sender<Message>,

    level: Level
}

impl MssqlDbLogger {
    /// create a new database logger
    pub fn new(app: &String, client: DbClient, level: Level) -> Self {
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(DbLoggerWorker::new(client, rx).run());

        Self { app: app.clone(), tx, level }
    }
}

impl log::Log for MssqlDbLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if let Some(msg) = record.args().as_str() {
            let msg = Message {
                timestamp: Local::now().naive_local(),
                app: self.app.clone(),
                level: record.level(),
                message: String::from(msg)
            };

            let tx = self.tx.clone();
            tokio::spawn(async move { tx.send(msg).await });
        }
    }

    fn flush(&self) {}
}

#[derive(Debug)]
struct DbLoggerWorker {
    client: DbClient,
    rx: Receiver<Message>
}

impl DbLoggerWorker {
    pub fn new(client: DbClient, rx: Receiver<Message>) -> Self {
        Self { client, rx }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            // we will just ignore failures
            let _ = self.client.execute(
                "INSERT INTO HighSteel.Log(timestamp, app, level, message) VALUES(@P1, @P2, @P3, @P4)",
                &[&msg.timestamp, &msg.app, &msg.level.as_str(), &msg.message]
            ).await;
        }
    }
}

#[derive(Debug)]
struct Message {
    timestamp: NaiveDateTime,
    app: String,
    level: Level,
    message: String
}
