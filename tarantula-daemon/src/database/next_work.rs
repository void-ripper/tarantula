use std::{collections::HashMap, sync::Arc};

use mcriddle::PubKeyBytes;
use sqlx::{Row, SqlitePool};
use tokio::sync::{oneshot, Mutex};
use url::Url;

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
        ORDER BY l.last_check
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

        let mut url = format!(
            "{}://{}{}",
            if https == 1 { "https" } else { "http" },
            host,
            path,
        );

        if !query.is_empty() {
            url.push('?');
            url.push_str(&query);
        }

        Ok(Command::ClaimWork { pubkey, url, oid })
    }

    pub async fn handle_claim_work(
        pool: &SqlitePool,
        claimers: &Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
        pubkey: PubKeyBytes,
        oid: String,
        url: String,
    ) -> Result<(), Error> {
        let purl = ex!(Url::parse(&url));
        let https = "https" == purl.scheme();
        let host = purl.host_str().unwrap();
        let path = purl.path();
        let query = purl.query().unwrap_or("");

        let sql = r#"
            UPDATE link SET last_check = current_timestamp
            WHERE
                host_id = (SELECT id FROM host WHERE https = $1 AND name = $2)
                AND path_id = (SELECT id FROM path WHERE path = $3)
                AND query_id = (SELECT id FROM query WHERE query = $4)
        "#;
        ex!(sqlx::query(sql)
            .bind(https)
            .bind(host)
            .bind(path)
            .bind(query)
            .execute(pool)
            .await);

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

    pub async fn scrap_result(
        &self,
        url: String,
        mut keywords: HashMap<String, u32>,
        links: Vec<String>,
    ) -> Result<(), Error> {
        let purl = ex!(Url::parse(&url));
        let https = "https" == purl.scheme();
        let host = purl.host_str().unwrap();
        let path = purl.path();
        let query = purl.query().unwrap_or("");

        let sql = r#"
        SELECT l.id FROM link l
        WHERE l.host_id = (SELECT id FROM host WHERE https = $1 AND name = $2)
            AND l.path_id = (SELECT id FROM path WHERE path = $3)
            AND l.query_id = (SELECT id FROM query WHERE query = $4)
        "#;
        let res = ex!(sqlx::query(sql)
            .bind(https)
            .bind(host)
            .bind(path)
            .bind(query)
            .fetch_one(&self.pool)
            .await);
        let lid: i64 = res.get(0);

        for (keyword, count) in keywords.drain() {
            let res = ex!(sqlx::query("SELECT id FROM keyword WHERE name = $1")
                .bind(&keyword)
                .fetch_optional(&self.pool)
                .await);
            let kid: i64 = if let Some(row) = res {
                row.get(0)
            } else {
                let res = ex!(
                    sqlx::query("INSERT INTO keyword(name) VALUES($1) RETURNING id")
                        .bind(&keyword)
                        .fetch_one(&self.pool)
                        .await
                );
                res.get(0)
            };
            let sql ="INSERT INTO link_keyword(link_id, keyword_id, count) VALUES($1, $2, $3) ON CONFLICT UPDATE count = $3";
            ex!(sqlx::query(sql)
                .bind(lid)
                .bind(kid)
                .bind(count)
                .execute(&self.pool)
                .await);
        }

        Ok(())
    }
}
