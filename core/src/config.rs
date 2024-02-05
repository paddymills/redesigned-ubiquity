
//! db configurations from local `.toml` files

use std::{
    fs::{self, File},
    path::PathBuf,
    io::Write
};

/// trait to allow for easy serde of a config
pub trait TomlConfig {
    /// load the configuration from `config.toml` in the root directory
    fn load<T: Into<PathBuf>>(path: T) -> anyhow::Result<Self>
        where Self: Sized + serde::de::DeserializeOwned
    {
        // read file
        let toml_contents = fs::read_to_string(path.into())?;

        // parse toml file text
        let parsed = toml::from_str::<Self>(&toml_contents)?;

        Ok(parsed)
    }

    /// generate an example config
    fn generate<T: Into<PathBuf>>(path: T) -> anyhow::Result<()>
        where Self: Sized + Default + serde::Serialize
    {
        let toml = toml::to_string(&Self::default())?;
        
        let mut file = File::create(path.into())?;
        file.write_all(toml.as_bytes())?;

        Ok(())
    }
}
