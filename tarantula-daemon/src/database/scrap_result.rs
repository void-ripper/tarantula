use std::collections::HashMap;

use sqlx::{Row, SqlitePool};
use url::Url;

use crate::{error::Error, ex};

use super::{add_url, Command, Database};

async fn get_link_id(pool: &SqlitePool, url: String) -> Result<i64, Error> {
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
        .fetch_one(pool)
        .await);
    Ok(res.get(0))
}

async fn get_link_id_or_insert(pool: &SqlitePool, url: String) -> Result<i64, Error> {
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
        .fetch_optional(pool)
        .await);

    if let Some(row) = res {
        Ok(row.get(0))
    } else {
        let hid = ex!(add_url::select_or_insert_host(pool, host, https).await);
        let pid = ex!(add_url::select_or_insert(pool, "path", "path", path).await);
        let fid = ex!(add_url::select_or_insert(pool, "query", "query", query).await);

        let sql = r#"
            INSERT INTO link(host_id, path_id, query_id) VALUES($1, $2, $3)
            ON CONFLICT (host_id, path_id, query_id) DO NOTHING
            RETURNING id
        "#;
        let res = ex!(sqlx::query(sql)
            .bind(hid)
            .bind(pid)
            .bind(fid)
            .fetch_one(pool)
            .await);

        Ok(res.get(0))
    }
}

impl Database {
    pub async fn handle_scrap_result(
        pool: &SqlitePool,
        url: String,
        mut keywords: HashMap<String, u32>,
        links: Vec<String>,
    ) -> Result<(), Error> {
        let lid = get_link_id(pool, url).await?;

        for (keyword, count) in keywords.drain() {
            let res = ex!(sqlx::query("SELECT id FROM keyword WHERE name = $1")
                .bind(&keyword)
                .fetch_optional(pool)
                .await);
            let kid: i64 = if let Some(row) = res {
                row.get(0)
            } else {
                let res = ex!(
                    sqlx::query("INSERT INTO keyword(name) VALUES($1) RETURNING id")
                        .bind(&keyword)
                        .fetch_one(pool)
                        .await
                );
                res.get(0)
            };
            let sql ="INSERT INTO link_keyword(link_id, keyword_id, count) VALUES($1, $2, $3) ON CONFLICT DO UPDATE SET count = $3";
            ex!(sqlx::query(sql)
                .bind(lid)
                .bind(kid)
                .bind(count)
                .execute(pool)
                .await);
        }

        for link in links {
            let id = get_link_id_or_insert(pool, link).await?;
            ex!(sqlx::query(
                "INSERT INTO link_to(link_id, to_id) VALUES($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(lid)
            .bind(id)
            .execute(pool)
            .await);
        }
        Ok(())
    }

    pub async fn scrap_result(
        &self,
        url: String,
        keywords: HashMap<String, u32>,
        links: Vec<String>,
    ) -> Result<(), Error> {
        let data = ex!(borsh::to_vec(&Command::ScrapResult {
            url,
            keywords,
            links
        }));

        ex!(self.peer.share(data).await);

        Ok(())
    }
}
