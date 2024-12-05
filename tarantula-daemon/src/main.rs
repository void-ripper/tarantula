use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use crate::error::Error;
use axum::{
    extract::ws::{Message, WebSocket},
    routing::any,
    Router,
};
use database::Database;
use futures_util::stream::SplitSink;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

mod config;
mod database;
mod error;
mod routes;
mod ws;

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

pub(crate) struct Client {
    out: SplitSink<WebSocket, Message>,
}

pub(crate) struct App {
    clients: RwLock<HashMap<SocketAddr, Arc<Client>>>,
    db: Database,
}

pub(crate) type AppPtr = Arc<App>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cfg = ex!(config::load("tarantula.toml"));

    tracing_subscriber::fmt().with_env_filter(&cfg.log).init();

    let app = Arc::new(App {
        clients: RwLock::new(HashMap::new()),
        db: ex!(Database::new(&cfg).await),
    });

    let route = Router::new()
        .nest("/", routes::config())
        .route("/ws", any(ws::handle_connection))
        .with_state(app.clone())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    tracing::info!("listen to: {}", cfg.listen);
    let listener = ex!(TcpListener::bind(cfg.listen).await);
    ex!(axum::serve(listener, route)
        .with_graceful_shutdown(async {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::error!("{e}");
            }
        })
        .await);

    Ok(())
}
