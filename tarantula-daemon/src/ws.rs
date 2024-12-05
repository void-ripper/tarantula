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

use crate::AppPtr;

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TarantulaMessage {
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

pub struct Client {
    out: SplitSink<WebSocket, Message>,
}

pub async fn handle_connection(
    State(app): State<AppPtr>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let (outgoing, mut incoming) = socket.split();
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
    })
}

pub async fn handle_message(
    app: &AppPtr,
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
