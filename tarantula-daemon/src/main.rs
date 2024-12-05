use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use crate::error::Error;
use database::Database;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
enum TarantulaMessage {
    AddUrl {
        url: String,
    },
    NextWork,
    NextWorkAnswer {
        url: String,
    },
    ScrapResult {
        url: String,
        keywords: HashMap<String, u32>,
        links: Vec<String>,
    },
}

struct Client {
    out: SplitSink<WebSocketStream<TcpStream>, Message>,
}

struct App {
    clients: RwLock<HashMap<SocketAddr, Arc<Client>>>,
    db: Database,
}

async fn handle_connection(app: Arc<App>, sck: TcpStream, addr: SocketAddr) -> Result<(), Error> {
    let ws_stream = ex!(tokio_tungstenite::accept_async(sck).await);

    let (outgoing, mut incoming) = ws_stream.split();
    let client = Arc::new(Client { out: outgoing });

    let app0 = app.clone();
    let client0 = client.clone();
    tokio::spawn(async move {
        loop {
            match incoming.next().await {
                Some(Ok(n)) => match n {
                    Message::Text(txt) => {
                        let msg: Result<TarantulaMessage, _> = serde_json::from_str(&txt);
                        handle_message(&app0, &client0, msg).await;
                    }
                    Message::Binary(bin) => {
                        let msg: Result<TarantulaMessage, _> = serde_json::from_slice(&bin);
                        handle_message(&app0, &client0, msg).await;
                    }
                    Message::Ping(_ping) => {
                        // outgoing.send(Message::Pong(ping)).await;
                    }
                    Message::Close(_cl) => break,
                    _ => {}
                },
                Some(Err(e)) => {
                    tracing::error!("{e}");
                }
                None => break,
            }
        }

        app0.clients.write().await.remove(&addr);
    });

    app.clients.write().await.insert(addr, client);

    Ok(())
}

async fn handle_message(
    app: &Arc<App>,
    client: &Arc<Client>,
    msg: Result<TarantulaMessage, serde_json::Error>,
) {
    match msg {
        Ok(msg) => match msg {
            TarantulaMessage::AddUrl { url } => {
                if let Err(e) = app.db.add_url(url).await {
                    tracing::error!("{e}");
                }
            }
            TarantulaMessage::NextWork => {}
            TarantulaMessage::NextWorkAnswer { .. } => {
                tracing::error!("we should never get a NextWorkAnswer");
            }
            TarantulaMessage::ScrapResult {
                url,
                keywords,
                links,
            } => {}
        },
        Err(e) => {
            tracing::error!("{e}");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cfg = ex!(config::load("tarantula.toml"));

    tracing_subscriber::fmt().with_env_filter(&cfg.log).init();

    let listener = ex!(TcpListener::bind(cfg.listen).await);

    let app = Arc::new(App {
        clients: RwLock::new(HashMap::new()),
        db: ex!(Database::new(&cfg).await),
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
