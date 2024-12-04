use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use crate::error::Error;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

mod config;
mod database;
mod error;

#[macro_export]
macro_rules! ex {
    ($e: expr) => {
        $e.map_err(|e| Error {
            line: line!(),
            module: module_path!().into(),
            msg: e.to_string(),
        })?
    };
}

struct Client {
    out: SplitSink<WebSocketStream<TcpStream>, Message>,
}

struct App {
    clients: RwLock<HashMap<SocketAddr, Client>>,
}

async fn handle_connection(app: Arc<App>, sck: TcpStream, addr: SocketAddr) -> Result<(), Error> {
    let ws_stream = ex!(tokio_tungstenite::accept_async(sck).await);

    let (outgoing, mut incoming) = ws_stream.split();

    let app0 = app.clone();
    tokio::spawn(async move {
        loop {
            match incoming.next().await {
                Some(Ok(n)) => {}
                Some(Err(e)) => {
                    tracing::error!("{e}");
                }
                None => break,
            }
        }

        app0.clients.write().await.remove(&addr);
    });

    app.clients
        .write()
        .await
        .insert(addr, Client { out: outgoing });

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cfg = ex!(config::load("tarantula.toml"));
    let listener = ex!(TcpListener::bind(cfg.listen).await);

    let app = Arc::new(App {
        clients: RwLock::new(HashMap::new()),
    });

    loop {
        match listener.accept().await {
            Ok((sck, addr)) => {
                if let Err(e) = handle_connection(app.clone(), sck, addr).await {
                    tracing::error!("{e}");
                }
            }
            Err(e) => {
                return Err(Error {
                    line: line!(),
                    module: module_path!().into(),
                    msg: e.to_string(),
                });
            }
        }
    }

    // Ok(())
}
