
//! db configurations from local `.toml` files

use std::{
    fs::{self, File},
    path::PathBuf,
    io::Write
};

/// trait to allow for easy serde of a config
pub trait Config {
    /// load the configuration from `config.toml` in the root directory
    fn load(path: &PathBuf) -> anyhow::Result<Self>
        where Self: Sized + serde::de::DeserializeOwned
    {
        // read file
        let toml_contents = fs::read_to_string(path)?;

        // parse toml file text
        let parsed = toml::from_str::<Self>(&toml_contents)?;

        Ok(parsed)
    }

    /// generate an example config
    fn generate(path: &PathBuf) -> anyhow::Result<()>
        where Self: Sized + Default + serde::Serialize
    {
        let toml = toml::to_string(&Self::default())?;
        
        let mut file = File::create(path)?;
        file.write_all(toml.as_bytes())?;

        Ok(())
    }
}
