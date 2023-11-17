
//! database connections

use tiberius::{AuthMethod, Client, Config, error::Error};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

/// Client type for SQL Server database
pub type DbClient = Client<Compat<TcpStream>>;
/// Result type for SQL Server database
pub type DbResult<T> = Result<T, Error>;

/// configure database connection
pub async fn connect(host: &str, database: &str) -> Result<DbClient, Error> {
    let mut config = Config::new();

    config.host(host);
    config.database(database);

    // use windows authentication
    config.authentication(AuthMethod::Integrated);
    config.trust_cert();

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let client = Client::connect(config, tcp.compat_write()).await?;

    Ok(client)
}

