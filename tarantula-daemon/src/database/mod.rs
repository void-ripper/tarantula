use std::{path::PathBuf, sync::Arc, time::Duration};

use borsh::{BorshDeserialize, BorshSerialize};
use futures_util::StreamExt;
use sqlx::{Executor, SqlitePool};

use crate::{config::Config, error::Error, ex};

mod add_url;
mod next_work;

#[derive(BorshDeserialize, BorshSerialize)]
pub(crate) enum Command {
    AddUrl { url: String },
    NextWork,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub(crate) enum Response {
    NextWork,
}

pub struct Database {
    peer: Arc<mcriddle::Peer>,
    pool: SqlitePool,
}

impl Database {
    pub async fn new(cfg: &Config) -> Result<Self, Error> {
        let pcfg = mcriddle::Config {
            addr: cfg.peer,
            folder: cfg.folder.clone(),
            keep_alive: Duration::from_millis(300),
            data_gather_time: Duration::from_millis(500),
            thin: false,
            relationship_time: Duration::from_secs(30),
            relationship_count: 3,
        };
        let peer = ex!(mcriddle::Peer::new(pcfg));
        let mut next_blk = peer.last_block_receiver();

        let folder = ex!(cfg.folder.canonicalize());
        let dbfile = folder.join("tarantula.db");
        tracing::info!("use db: {}", dbfile.display());

        let existed = dbfile.exists();

        if !existed {
            ex!(std::fs::write(&dbfile, ""));
        }

        let pool = ex!(SqlitePool::connect(&dbfile.to_string_lossy()).await);
        let pool0 = pool.clone();

        if !existed {
            let mut stream = pool.execute_many(include_str!("../schema.sql"));
            while let Some(Ok(_n)) = stream.next().await {}
        }

        tokio::spawn(async move {
            loop {
                match next_blk.recv().await {
                    Ok(blk) => {
                        for data in blk.data {
                            match borsh::from_slice::<Command>(&data.data) {
                                Ok(cmd) => {
                                    let res = match cmd {
                                        Command::AddUrl { url } => {
                                            Self::handle_add_url(&pool0, url).await
                                        }
                                        Command::NextWork => Self::handle_next_work(&pool0).await,
                                    };

                                    if let Err(e) = res {
                                        tracing::error!("command error: {e}");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("borsh: {e}");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("{e}");
                    }
                }
            }
        });

        Ok(Self { peer, pool })
    }
}