use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{error::Error, ex, AppPtr};

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TarantulaMessage {
    AddUrl {
        url: String,
    },
    NextWork {
        pubkey: String,
    },
    NextWorkAnswer {
        url: String,
    },
    ScrapResult {
        url: String,
        keywords: HashMap<String, u32>,
        links: Vec<String>,
    },
}

pub struct Client {
    out: Mutex<SplitSink<WebSocket, Message>>,
}

pub async fn handle_connection(
    State(app): State<AppPtr>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let (outgoing, mut incoming) = socket.split();
        let client = Arc::new(Client {
            out: Mutex::new(outgoing),
        });

        let app0 = app.clone();
        let client0 = client.clone();
        tokio::spawn(async move {
            loop {
                match incoming.next().await {
                    Some(Ok(n)) => match n {
                        Message::Text(txt) => {
                            let msg: Result<TarantulaMessage, _> = serde_json::from_str(&txt);
                            if let Err(e) = handle_message(&app0, &client0, msg).await {
                                tracing::error!("ws message: {e}");
                            }
                        }
                        Message::Binary(bin) => {
                            let msg: Result<TarantulaMessage, _> = serde_json::from_slice(&bin);
                            if let Err(e) = handle_message(&app0, &client0, msg).await {
                                tracing::error!("ws message: {e}");
                            }
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
    })
}

pub async fn handle_message(
    app: &AppPtr,
    client: &Arc<Client>,
    msg: Result<TarantulaMessage, serde_json::Error>,
) -> Result<(), Error> {
    let msg = ex!(msg);
    match msg {
        TarantulaMessage::AddUrl { url } => {
            ex!(app.db.add_url(url).await);
        }
        TarantulaMessage::NextWork { pubkey } => {
            let pubk = hex::decode(pubkey).unwrap();
            let mut pubkey = [0u8; 33];
            pubkey.copy_from_slice(&pubk);

            let url = ex!(app.db.get_next_work(pubkey).await);
            let msg = TarantulaMessage::NextWorkAnswer { url };
            let data = ex!(serde_json::to_string(&msg));
            ex!(client.out.lock().await.send(Message::Text(data)).await);
        }
        TarantulaMessage::NextWorkAnswer { .. } => {
            tracing::error!("we should never get a NextWorkAnswer");
        }
        TarantulaMessage::ScrapResult {
            url,
            keywords,
            links,
        } => {}
    }

    Ok(())
}
