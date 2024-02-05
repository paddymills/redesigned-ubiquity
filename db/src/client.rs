
//! Database client

use serde::{Deserialize, Serialize};
use tiberius::{AuthMethod, Client, Config, error::Error};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

/// Client type for SQL Server database
pub type DbClient = Client<Compat<TcpStream>>;
/// Result type for SQL Server database
pub type DbResult<T> = Result<T, Error>;

/// Parameters for a SQL Server connection
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DbConnParams {
    /// Server name
    pub server: String,
    
    /// Database name
    pub database: String,
}

impl DbConnParams {
    /// connect to the database using the configuration
    pub async fn connect(&self) -> DbResult<DbClient> {
        connect(&self.server, &self.database).await
    }
}

impl sysinteg_core::config::TomlConfig for DbConnParams {}

impl Default for DbConnParams {
    fn default() -> Self {
        Self {
            server: String::from("<server>"),
            database: String::from("<database>")
        }
    }
}

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
