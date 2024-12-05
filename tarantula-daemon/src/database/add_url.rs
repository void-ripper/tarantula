use sqlx::{Row, SqlitePool};
use url::Url;

use crate::{database::Command, error::Error, ex};

use super::Database;

impl Database {
    pub(crate) async fn handle_add_url(pool: &SqlitePool, url: String) -> Result<(), Error> {
        let url = ex!(Url::parse(&url));

        let host = url.host_str().unwrap_or("");
        let path = url.path();

        // let mut db = ex!(pool.acquire().await);
        let sql = r#"
            INSERT INTO host(name) VALUES ($1)
            ON CONFLICT (name) DO NOTHING
            RETURNING id
        "#;
        let row = ex!(sqlx::query(sql).bind(host).fetch_one(pool).await);
        let hid: i64 = row.get(0);

        let sql = r#"
            INSERT INTO path(path) VALUES($1)
            ON CONFLICT (path) DO NOTHING
            RETURNING id
        "#;
        let row = ex!(sqlx::query(sql).bind(path).fetch_one(pool).await);
        let pid: i64 = row.get(0);

        let sql = r#"
            INSERT INTO host_path(host_id, path_id) VALUES($1, $1)
            ON CONFLICT (host_id, path_id) DO NOTHING
        "#;
        ex!(sqlx::query(sql).bind(hid).bind(pid).execute(pool).await);

        Ok(())
    }

    pub async fn add_url(&self, url: String) -> Result<(), Error> {
        let data = ex!(borsh::to_vec(&Command::AddUrl { url }));
        ex!(self.peer.share(data).await);

        Ok(())
    }
}
