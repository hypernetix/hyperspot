use crate::{ConnectOpts, DbHandle, Result};
use sea_orm::DatabaseConnection;

/// Setup a new in-memory `SQLite` database for testing
pub async fn setup_sqlite_db() -> Result<(DbHandle, DatabaseConnection)> {
    let db = DbHandle::connect("sqlite::memory:", ConnectOpts::default()).await?;
    let conn = db.sea_secure().conn().clone();
    Ok((db, conn))
}
