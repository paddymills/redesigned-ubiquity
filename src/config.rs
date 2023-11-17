
//! db configurations from local `.toml` files

use crate::db;
use std::fs;
use serde::{Deserialize, Serialize};

/// Parameters for a SQL Server connection
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DbConnParams {
    /// Server name
    pub server: String,
    
    /// Database name
    pub database: String,
}

impl DbConnParams {
    /// load the configuration from `config.toml` in the root directory
    pub fn load() -> anyhow::Result<Self> {
        // read file
        let toml_contents = fs::read_to_string("config.toml")?;

        // parse toml file text
        let parsed = toml::from_str::<Self>(&toml_contents)?;

        Ok(parsed)
    }

    /// connect to the database using the configuration
    pub async fn connect(&self) -> db::DbResult<db::DbClient> {
        db::connect(&self.server, &self.database).await
    }
}
