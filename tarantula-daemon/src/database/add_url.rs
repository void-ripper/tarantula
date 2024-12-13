use sqlx::{Row, SqlitePool};
use url::Url;

use crate::{database::Command, error::Error, ex};

use super::Database;

pub async fn select_or_insert(
    pool: &SqlitePool,
    name: &str,
    col: &str,
    val: &str,
) -> Result<i64, Error> {
    let select = format!("SELECT id FROM {} WHERE {} = $1", name, col);
    let id = ex!(sqlx::query(&select).bind(val).fetch_optional(pool).await);

    Ok(if let Some(id) = id {
        id.get(0)
    } else {
        let insert = format!("INSERT INTO {}({}) VALUES($1) RETURNING id", name, col);
        let res = ex!(sqlx::query(&insert).bind(val).fetch_one(pool).await);
        res.get(0)
    })
}

pub async fn select_or_insert_host(
    pool: &SqlitePool,
    val: &str,
    https: bool,
) -> Result<i64, Error> {
    let select = "SELECT id FROM host WHERE https = $1 AND name = $2";
    let id = ex!(sqlx::query(select)
        .bind(https)
        .bind(val)
        .fetch_optional(pool)
        .await);

    Ok(if let Some(id) = id {
        id.get(0)
    } else {
        let insert = "INSERT INTO host(https, name) VALUES($1, $2) RETURNING id";
        let res = ex!(sqlx::query(insert)
            .bind(https)
            .bind(val)
            .fetch_one(pool)
            .await);
        res.get(0)
    })
}

impl Database {
    pub async fn handle_add_url(pool: &SqlitePool, url: String) -> Result<(), Error> {
        let url = ex!(Url::parse(&url));

        let https = url.scheme() == "https";
        let host = url.host_str().unwrap_or("");
        let path = url.path();
        let query = url.query().unwrap_or("");

        let hid = ex!(select_or_insert_host(pool, host, https).await);
        let pid = ex!(select_or_insert(pool, "path", "path", path).await);
        let fid = ex!(select_or_insert(pool, "query", "query", query).await);

        let sql = r#"
            INSERT INTO link(host_id, path_id, query_id) VALUES($1, $2, $3)
            ON CONFLICT (host_id, path_id, query_id) DO NOTHING
        "#;
        ex!(sqlx::query(sql)
            .bind(hid)
            .bind(pid)
            .bind(fid)
            .execute(pool)
            .await);

        Ok(())
    }

    pub async fn add_url(&self, url: String) -> Result<(), Error> {
        let purl = ex!(Url::parse(&url));
        let https = match purl.scheme() {
            "https" => true,
            "http" => false,
            n => {
                return Err(Error {
                    line: line!(),
                    module: module_path!().into(),
                    msg: format!("not supported schema: {}", n),
                })
            }
        };
        let host = purl.host_str().unwrap();
        let path = purl.path();
        let query = purl.query().unwrap_or("");

        let sql = r#"
            SELECT l.id FROM host h, path p, query f, link l
            WHERE l.host_id = h.id AND l.path_id = p.id AND l.query_id = f.id
                AND h.https = $1 AND h.name = $2 AND p.path = $3 AND f.query = $4
        "#;
        let res = ex!(sqlx::query(sql)
            .bind(https)
            .bind(host)
            .bind(path)
            .bind(query)
            .fetch_optional(&self.pool)
            .await);

        if res.is_none() {
            let data = ex!(borsh::to_vec(&Command::AddUrl { url }));
            ex!(self.peer.share(data).await);
        }

        Ok(())
    }
}
