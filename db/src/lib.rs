
//! database connections

use serde::{Deserialize, Serialize};
use tiberius::{AuthMethod, Client, Config, error::Error, ColumnData, Row};
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

/// convert SQL Server column to string, regardless of datatype
pub fn column_to_str(column: ColumnData) -> String {
    use ColumnData::*;
    match column {
        U8(n)  => n.unwrap().to_string(),
        I16(n) => n.unwrap().to_string(),
        I32(n) => n.unwrap().to_string(),
        I64(n) => n.unwrap().to_string(),
        F32(n) => n.unwrap().to_string(),
        F64(n) => n.unwrap().to_string(),
        Numeric(n) => n.unwrap().to_string(),

        String(s) => s.unwrap().to_string(),

        _ => unimplemented!()
    }
}

/// Converts a SQL row to a tab-delimited string
pub fn row_to_string(row: Row) -> String {
    row.into_iter()
        .map(column_to_str)

        // generally speaking, one the these lines should be no more than 128 characters
        //  so we set an initial capacity of 128 to try to avoid reallocations
        .fold(String::with_capacity(128), |acc, s| acc + &s + "\t")
        
        .trim_end() // remove trailing '\t'
        .into()
}

