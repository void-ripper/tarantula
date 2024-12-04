use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use crate::{error::Error, ex};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub listen: SocketAddr,
    pub folder: PathBuf,
}

pub fn load<P: AsRef<Path>>(name: P) -> Result<Config, Error> {
    let data = ex!(std::fs::read_to_string(name));
    Ok(ex!(toml::from_str(&data)))
}
