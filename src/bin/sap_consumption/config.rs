
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use sysinteg::config::Config;
use sysinteg::db::DbConnParams;

pub const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Deserialize, Serialize)]
pub struct SapConsumptionConfig {
    pub database: DbConnParams,
    pub output_dir: PathBuf
}

impl Config for SapConsumptionConfig {}

impl Default for SapConsumptionConfig {
    fn default() -> Self {
        Self {
            database: DbConnParams::default(),
            output_dir: PathBuf::from(r"\\<server>\<path to where .ready files are placed>"),
        }
    }
}
