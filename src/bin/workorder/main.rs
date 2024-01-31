
use clap::{Parser, Subcommand};
use pretty_env_logger;

use sysinteg::api::JobShipment;

/// Work order management system
#[derive(Debug, Parser)]
#[clap(name = "Workorder")]
#[clap(author, version)]
struct Cli {
    /// Subcommand to run
    #[clap(subcommand)]
    command: Commands,

    /// verbosity level
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// update work order
    Update {
        /// Job number (with structure letter) and shipment
        job_shipment: JobShipment,
    },

    /// check for jobs to update from SAP load files
    CheckUpdate,
}

impl Commands {
    fn handle_command(self) {
        match self {
            Self::Update { job_shipment } => println!("Updating {}...", job_shipment),
            Self::CheckUpdate => println!("Checking for updates..."),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    
    let args = Cli::parse();
    log::debug!("{:#?}", args);
    args.command.handle_command();

    Ok(())
}
