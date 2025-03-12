use std::{
    error::Error,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub browser: PathBuf,
    pub api: String,
    /// The log filter.
    pub log: String,
    pub parallel: u32,
}

pub fn load<P: AsRef<Path>>(name: P) -> Result<Config, Box<dyn Error>> {
    let data = std::fs::read_to_string(name)?;
    Ok(toml::from_str(&data)?)
}
