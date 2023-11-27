
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use sysinteg::config::TomlConfig;
use sysinteg::db::DbConnParams;

pub const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Deserialize, Serialize)]
pub struct SapConsumptionConfig {
    pub database: DbConnParams,
    pub output_dir: PathBuf,
    pub logging_name: String,
}

impl TomlConfig for SapConsumptionConfig {}

impl Default for SapConsumptionConfig {
    fn default() -> Self {
        Self {
            database: DbConnParams::default(),
            output_dir: PathBuf::from(r"\\<server>\<path to where .ready files are placed>"),
            logging_name: String::from("<application name used for logging to the Windows Event Log>")
        }
    }
}
