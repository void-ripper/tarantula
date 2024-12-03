use std::{fmt::Display, net::SocketAddr, path::Path};

use serde::Deserialize;
use tokio::net::TcpListener;

#[derive(Debug)]
struct Error {
    pub line: u32,
    pub module: String,
    pub msg: String,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}] => {}", self.module, self.line, self.msg)
    }
}

macro_rules! ex {
    ($e: expr) => {
        $e.map_err(|e| Error {
            line: line!(),
            module: module_path!().into(),
            msg: e.to_string(),
        })?
    };
}

#[derive(Deserialize)]
struct Config {
    pub listen: SocketAddr,
}

 fn load<P: AsRef<Path>>(name: P) -> Result<Config, Error> {
    let data = ex!(std::fs::read_to_string(name));
    Ok(ex!(toml::from_str(&data)))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cfg = ex!(load("tarantula.toml"));
    let listener = ex!(TcpListener::bind(cfg.listen).await);

    listener.accept()

    Ok(())
}
