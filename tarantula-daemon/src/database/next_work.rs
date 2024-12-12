use std::{collections::HashMap, sync::Arc};

use mcriddle::PubKeyBytes;
use sqlx::{Row, SqlitePool};
use tokio::sync::{oneshot, Mutex};

use crate::{error::Error, ex};

use super::{Command, Database};

impl Database {
    pub async fn handle_next_work(
        pool: &SqlitePool,
        pubkey: PubKeyBytes,
        oid: String,
    ) -> Result<Command, Error> {
        let sql = r#"
        SELECT l.id, h.https, h.name, p.path, q.query FROM link l
        LEFT JOIN host h ON l.host_id = h.id
        LEFT JOIN path p ON l.path_id = p.id
        LEFT JOIN query q ON l.query_id = q.id
        ORDER l.last_check
        LIMIT 1
        "#;
        let res = ex!(sqlx::query(sql).fetch_one(pool).await);

        let lid: i64 = res.get(0);
        let https: i64 = res.get(1);
        let host: String = res.get(2);
        let path: String = res.get(3);
        let query: String = res.get(4);

        ex!(
            sqlx::query("UPDATE link SET last_check = current_timestamp WHERE id = $1")
                .bind(lid)
                .execute(pool)
                .await
        );

        let url = format!(
            "{}://{}{}{}",
            if https == 1 { "https" } else { "http" },
            host,
            path,
            query
        );

        Ok(Command::ClaimWork { pubkey, url, oid })
    }

    pub async fn handle_claim_work(
        claimers: &Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
        pubkey: PubKeyBytes,
        oid: String,
        url: String,
    ) -> Result<(), Error> {
        if let Some(tx) = claimers.lock().await.remove(&oid) {
            ex!(tx.send(url));
        }

        Ok(())
    }

    pub async fn get_next_work(&self, pubkey: PubKeyBytes) -> Result<String, Error> {
        let (tx, rx) = oneshot::channel();
        let oid = nanoid::nanoid!(12);
        let data = ex!(borsh::to_vec(&Command::NextWork {
            pubkey,
            oid: oid.clone()
        }));

        self.claimers.lock().await.insert(oid, tx);

        ex!(self.peer.share(data).await);

        let url = ex!(rx.await);

        Ok(url)
    }
}
