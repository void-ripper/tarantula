use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use crate::error::Error;
use axum::Router;
use clap::Parser;
use database::Database;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

mod config;
mod database;
mod error;
mod routes;

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

pub(crate) struct App {
    db: Database,
}

pub(crate) type AppPtr = Arc<App>;

#[derive(Parser)]
#[command(about, author, version)]
struct Args {
    #[arg(long)]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let cfg = ex!(config::load(args.config));

    tracing_subscriber::fmt().with_env_filter(&cfg.log).init();

    if !cfg.folder.exists() {
        ex!(std::fs::create_dir_all(&cfg.folder));
    }

    let app = Arc::new(App {
        db: ex!(Database::new(&cfg).await),
    });

    let route = Router::new()
        .merge(routes::config())
        .with_state(app.clone())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    tracing::info!("listen to: {}", cfg.listen);
    let listener = ex!(TcpListener::bind(cfg.listen).await);
    ex!(axum::serve(
        listener,
        route.into_make_service_with_connect_info::<SocketAddr>()
    )
    .with_graceful_shutdown(async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("{e}");
        }
    })
    .await);

    Ok(())
}
