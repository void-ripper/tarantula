use sqlx::SqlitePool;

use crate::error::Error;

use super::Database;

impl Database {
    pub(crate) async fn handle_next_work(pool: &SqlitePool) -> Result<(), Error> {
        Ok(())
    }

    pub async fn get_next_work(&self) -> Result<(), Error> {
        Ok(())
    }
}
