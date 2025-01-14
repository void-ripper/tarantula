use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use crate::{error::Error, ex};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    /// The web frontend and websocket api.
    pub listen: SocketAddr,
    /// The address of the mccloud node.
    pub peer: SocketAddr,
    /// A Socks5 proxy. Commonly used for TOR.
    pub proxy: Option<mccloud::config::Proxy>,
    /// The folder where the data is stored.
    pub folder: PathBuf,
    /// The log filter.
    pub log: String,
    /// Initial node connections.
    pub connections: Vec<String>,
}

pub fn load<P: AsRef<Path>>(name: P) -> Result<Config, Error> {
    let data = ex!(std::fs::read_to_string(name));
    Ok(ex!(toml::from_str(&data)))
}
