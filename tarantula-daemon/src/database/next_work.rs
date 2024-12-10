use sqlx::SqlitePool;

use crate::{error::Error, ex};

use super::Database;

impl Database {
    pub(crate) async fn handle_next_work(pool: &SqlitePool) -> Result<(), Error> {
        let sql = r#"
        SELECT h.name, p.path, q.query FROM link l
        LEFT JOIN host h ON l.host_id = h.id
        LEFT JOIN path p ON l.path_id = p.id
        LEFT JOIN query q ON l.query_id = q.id
        ORDER l.last_check
        LIMIT 1
        "#;
        let res = ex!(sqlx::query(sql).fetch_one(pool).await);

        Ok(())
    }

    pub async fn get_next_work(&self) -> Result<(), Error> {
        Ok(())
    }
}
