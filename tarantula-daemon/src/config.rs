use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use crate::{error::Error, ex};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    /// The websocket.
    pub listen: SocketAddr,
    pub peer: SocketAddr,
    pub folder: PathBuf,
    /// The log filter.
    pub log: String,
}

pub fn load<P: AsRef<Path>>(name: P) -> Result<Config, Error> {
    let data = ex!(std::fs::read_to_string(name));
    Ok(ex!(toml::from_str(&data)))
}
