#![allow(clippy::unwrap_used, clippy::expect_used)]

#[cfg(feature = "sqlite")]
mod sqlite_tx_tests {
    use modkit_db::{ConnectOpts, DbHandle};

    #[tokio::test]
    async fn sqlite_with_tx_commit_persists_changes() {
        let opts = ConnectOpts {
            max_conns: Some(1),
            ..Default::default()
        };
        let db = DbHandle::connect("sqlite::memory:", opts)
            .await
            .expect("Failed to connect to database");
        let pool = db.sqlx_sqlite().unwrap();

        sqlx::query("CREATE TABLE tx_test (id INTEGER PRIMARY KEY, val TEXT NOT NULL)")
            .execute(pool)
            .await
            .expect("Failed to create table");

        db.with_sqlite_tx(|tx| {
            Box::pin(async move {
                sqlx::query("INSERT INTO tx_test (id, val) VALUES (?, ?)")
                    .bind(1_i64)
                    .bind("committed")
                    .execute(&mut **tx)
                    .await?;
                Ok(())
            })
        })
        .await
        .expect("Transaction failed");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tx_test")
            .fetch_one(pool)
            .await
            .expect("Failed to query count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn sqlite_with_tx_error_rolls_back() {
        let opts = ConnectOpts {
            max_conns: Some(1),
            ..Default::default()
        };
        let db = DbHandle::connect("sqlite::memory:", opts)
            .await
            .expect("Failed to connect to database");
        let pool = db.sqlx_sqlite().unwrap();

        sqlx::query("CREATE TABLE tx_test (id INTEGER PRIMARY KEY, val TEXT NOT NULL)")
            .execute(pool)
            .await
            .expect("Failed to create table");

        let result: Result<(), _> = db
            .with_sqlite_tx(|tx| {
                Box::pin(async move {
                    sqlx::query("INSERT INTO tx_test (id, val) VALUES (?, ?)")
                        .bind(1_i64)
                        .bind("should_rollback")
                        .execute(&mut **tx)
                        .await?;
                    Err(modkit_db::DbError::Other(anyhow::anyhow!(
                        "Simulated error"
                    )))
                })
            })
            .await;

        assert!(result.is_err());

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tx_test")
            .fetch_one(pool)
            .await
            .expect("Failed to query count");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn sqlite_with_tx_returns_value() {
        let opts = ConnectOpts {
            max_conns: Some(1),
            ..Default::default()
        };
        let db = DbHandle::connect("sqlite::memory:", opts)
            .await
            .expect("Failed to connect to database");
        let pool = db.sqlx_sqlite().unwrap();

        sqlx::query("CREATE TABLE tx_test (id INTEGER PRIMARY KEY, val TEXT NOT NULL)")
            .execute(pool)
            .await
            .expect("Failed to create table");

        let inserted_id = db
            .with_sqlite_tx(|tx| {
                Box::pin(async move {
                    sqlx::query("INSERT INTO tx_test (id, val) VALUES (?, ?)")
                        .bind(42_i64)
                        .bind("test_value")
                        .execute(&mut **tx)
                        .await?;
                    Ok(42_i64)
                })
            })
            .await
            .expect("Transaction failed");

        assert_eq!(inserted_id, 42);

        let val: String = sqlx::query_scalar("SELECT val FROM tx_test WHERE id = ?")
            .bind(42_i64)
            .fetch_one(pool)
            .await
            .expect("Failed to query value");
        assert_eq!(val, "test_value");
    }
}
