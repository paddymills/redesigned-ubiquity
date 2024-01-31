
//! logging framework

use chrono::{Local, NaiveDateTime};
use eventlog::EventLog;
use log::{Level, Log, LevelFilter};

use tokio::sync::mpsc::{self, Sender, Receiver};
use tokio::task::JoinHandle;

use sysinteg_db::*;

const CONTROLLER_TARGET: &str = "LOGGING_CONTROLLER";

#[derive(Debug)]
enum Message {
    Shutdown,
    Message(LogMessage)
}

#[derive(Debug)]
struct LogMessage {
    timestamp: NaiveDateTime,
    app: String,
    level: Level,
    message: String
}

// TODO: move to sysinteg-db
/// Logger that logs to a MSSQL database
#[derive(Debug)]
pub struct MssqlDbLogger {
    tx: Sender<Message>,
    worker: Option<JoinHandle<()>>,

    level: Level
}

impl MssqlDbLogger {
    /// create a new database logger
    pub fn new(conn_params: &DbConnParams, level: Level) -> Self {
        let (tx, rx) = mpsc::channel(32);

        let params = conn_params.clone();
        let worker = Some(
            tokio::spawn(async move { DbLoggerWorker::run(params, rx).await })
        );

        Self { worker, tx, level }
    }

    /// takes the ['tokio::task::JoinHandle`] from the logger
    /// 
    /// *panics* if this is called more than once
    ///     as the worker can only be taken once (see [`Option::take`])
    pub fn take_worker(&mut self) -> JoinHandle<()> {
        self.worker.take()
            .ok_or(anyhow::anyhow!("Worker was already taken from MssqlDbLogger"))
            .unwrap()
    }
}

impl Log for MssqlDbLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        match format!("{}", record.args()) {
            s if s.as_str() == "<SHUTDOWN>" => {
                // in the event that someone decides to log "<SHUTDOWN>",
                //  we want to make sure that we only shut down if it came from EventAndDbLogger::finalize()
                if record.target() == CONTROLLER_TARGET {
                    let tx = self.tx.clone();
                    tokio::spawn(async move { tx.send(Message::Shutdown).await });
                }
            },
            msg => {
                let payload = Message::Message(LogMessage {
                    timestamp: Local::now().naive_local(),
                    app: record.target().into(),
                    level: record.level(),
                    message: msg
                });

                let tx = self.tx.clone();
                tokio::spawn(async move { tx.send(payload).await });
            }
        }
    }

    fn flush(&self) {}
}

#[derive(Debug)]
struct DbLoggerWorker {}

impl DbLoggerWorker {
    pub async fn run(params: DbConnParams, mut rx: Receiver<Message>) {
        let client = params.connect().await;

        match client {
            Ok(mut client) => {
                while let Some(msg) = rx.recv().await {
                    match msg {
                        Message::Shutdown => break,
                        Message::Message(msg) => {
                            // we will just ignore failures for simplicity
                            let _ = client.execute(
                                "INSERT INTO HighSteel.Log(timestamp, app, level, message) VALUES(@P1, @P2, @P3, @P4)",
                                &[&msg.timestamp, &msg.app, &msg.level.as_str(), &msg.message]
                            ).await;
                        }
                    }
                }
            },

            // although this won't hit the database logger, it might hit other active loggers
            _ => {
                std::thread::sleep(std::time::Duration::from_secs(2));    
                log::warn!("Failed to connect to database for logging")
            }
        }

        // we should close the channel for a cleaner shutdown in the case of
        //  - the client failed to connect at the beginning
        //  - Message::End was received
        //
        // any further writes to the channel will result in an error,
        //  but that is OK because we are ignoring errors on sending messages
        rx.close();     
    }
}

/// A logger that logs to the Windows Event log and a Database
pub struct EventAndDbLogger {
    db_worker: JoinHandle<()>
}

impl EventAndDbLogger {
    /// initialize the loggers
    pub async fn init<T, I>(name: &str, dbcfg: &DbConnParams, level: LevelFilter, addl_modules: T) -> anyhow::Result<Self>
        where
            I: ToString,
            T: IntoIterator<Item = I>
    {
        // we will use the the most verbose logging level because fern will handle level filtering
        let loggers_cfg_level = log::Level::max();

        // create loggers
        let event_logger = EventLog::new(name, loggers_cfg_level)?;
        let mut db_logger = MssqlDbLogger::new(dbcfg, loggers_cfg_level);

        // get handle to the `tokio::spawn` task MssqlDbLogger worker is running in
        let worker = db_logger.take_worker();

        // these have to be declared here with type annotations and not inline
        //  so that fern knows that they are `Box<dyn Log>`
        let event_logger: Box<dyn Log> = Box::new( event_logger );
        let db_logger: Box<dyn Log> = Box::new( db_logger );


        let mut logger = fern::Dispatch::new()
            // Dependencies such as Tokio/Tiberius do some logging of their own.
            //  This allows us to use a lower log level, while silencing their verbose logs.
            .level(log::LevelFilter::Error)
            .level_for("sysinteg", level);

        for module in addl_modules {
            logger = logger
                .level_for(module.to_string(), level);
        }

        logger
            // database logger (Actor pattern where database transactions happen on a separate thread)
            .chain(db_logger)

            // log to Windows Event Log
            .chain(
                fern::Dispatch::new()
                    // only Errors for the event log
                    .level(log::LevelFilter::Error)
                    .level_for(CONTROLLER_TARGET, LevelFilter::Off)
                    .chain(event_logger)
            )
            .apply()?;


        Ok( Self { db_worker: worker } )
    }

    /// final clean up of the logging system
    /// 
    /// This is needed to makes sure all logs get pushed to the database.
    /// Since the actual database logging is on a separate task/thread,
    /// if we let the program naturally exit, all messages in the queue and
    /// in process will not finish being pushed to the database. This blocks
    /// for the worker to finish before exiting.
    pub async fn finalize(self) {
        // ask the logger to tell the worker to shutdown
        log::error!(target: CONTROLLER_TARGET, "<SHUTDOWN>");

        // wait for the worker to finish logging to the database
        let _ = self.db_worker.await;
    }
}

