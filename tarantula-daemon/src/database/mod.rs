use std::{collections::HashMap, sync::Arc, time::Duration};

use borsh::{BorshDeserialize, BorshSerialize};
use futures_util::StreamExt;
use mcriddle::{blockchain::Data, PubKeyBytes, SignBytes};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    Executor, SqlitePool,
};
use tokio::sync::{broadcast, oneshot, Mutex};

use crate::{config::Config, error::Error, ex};

mod add_url;
mod next_work;
mod scrap_result;

#[derive(BorshDeserialize, BorshSerialize)]
pub(crate) enum Command {
    AddUrl {
        url: String,
    },
    NextWork {
        pubkey: PubKeyBytes,
        oid: String,
    },
    ClaimWork {
        pubkey: PubKeyBytes,
        oid: String,
        url: String,
    },
    ScrapResult {
        url: String,
        keywords: HashMap<String, u32>,
        links: Vec<String>,
    },
}

pub struct Database {
    peer: Arc<mcriddle::Peer>,
    claimers: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
    pool: SqlitePool,
}

impl Database {
    pub async fn new(cfg: &Config) -> Result<Self, Error> {
        let dbfile = cfg.folder.join("tarantula.db");
        tracing::info!("use db: {}", dbfile.display());

        let existed = dbfile.exists();

        let opts = SqliteConnectOptions::new()
            .filename(&dbfile)
            .foreign_keys(true)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);
        let pool = ex!(SqlitePool::connect_with(opts).await);
        let pool0 = pool.clone();

        if !existed {
            tracing::info!("initialize database");
            let mut stream = pool.execute_many(include_str!("../schema.sql"));
            while let Some(Ok(_n)) = stream.next().await {}
        }

        let pcfg = mcriddle::Config {
            addr: cfg.peer,
            folder: cfg.folder.clone(),
            keep_alive: Duration::from_millis(300),
            data_gather_time: Duration::from_millis(500),
            thin: false,
            relationship_time: Duration::from_secs(30),
            relationship_count: 3,
            next_candidates: 3,
        };
        let peer = ex!(mcriddle::Peer::new(pcfg));
        let next_blk = peer.last_block_receiver();

        for con in cfg.connections.iter() {
            if let Err(e) = peer.connect(*con).await {
                tracing::error!("{e}");
            }
        }

        let pool1 = pool.clone();
        let peer1 = peer.clone();
        peer.set_on_block_creation_cb(move |mut data| {
            let pool1 = pool1.clone();
            let peer1 = peer1.clone();

            Box::pin(async move {
                let mut to_add = Vec::new();

                for v in data.values() {
                    match borsh::from_slice(&v.data) {
                        Ok(cmd) => match cmd {
                            Command::NextWork { pubkey, oid } => {
                                let cmd = Self::handle_next_work(&pool1, pubkey, oid)
                                    .await
                                    .map_err(|e| {
                                        mcriddle::Error::external(
                                            line!(),
                                            module_path!(),
                                            e.to_string(),
                                        )
                                    })?;
                                let cmd_data = borsh::to_vec(&cmd)
                                    .map_err(|e| mcriddle::Error::io(line!(), module_path!(), e))?;
                                let new_data = peer1.create_data(cmd_data)?;
                                to_add.push(new_data);
                            }
                            _ => {}
                        },
                        Err(e) => {
                            tracing::error!("data parse error on block creation: {e}");
                        }
                    }
                }

                for d in to_add.drain(..) {
                    data.insert(d.sign, d);
                }

                Ok(data)
            })
        })
        .await;

        let claimers = Arc::new(Mutex::new(HashMap::new()));
        let claimers0 = claimers.clone();
        tokio::spawn(async move {
            Self::handle_new_blocks(next_blk, pool0, claimers0).await;
        });

        Ok(Self {
            peer,
            pool,
            claimers,
        })
    }

    async fn handle_new_blocks(
        mut next_blk: broadcast::Receiver<mcriddle::blockchain::Block>,
        pool: SqlitePool,
        claimers: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
    ) {
        loop {
            match next_blk.recv().await {
                Ok(blk) => {
                    for data in blk.data {
                        match borsh::from_slice::<Command>(&data.data) {
                            Ok(cmd) => {
                                let res = match cmd {
                                    Command::AddUrl { url } => {
                                        Self::handle_add_url(&pool, url).await
                                    }
                                    Command::NextWork { .. } => {
                                        // tracing::error!("NextWork should not be in a block");
                                        Ok(())
                                    }
                                    Command::ClaimWork { pubkey, oid, url } => {
                                        Self::handle_claim_work(&pool, &claimers, pubkey, oid, url)
                                            .await
                                    }
                                    Command::ScrapResult {
                                        url,
                                        keywords,
                                        links,
                                    } => {
                                        Self::handle_scrap_result(&pool, url, keywords, links).await
                                    }
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
    }
}
