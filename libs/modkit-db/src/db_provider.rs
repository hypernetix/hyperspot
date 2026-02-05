use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use crate::secure::{DbConn, DbTx};
use crate::{Db, DbError};

/// Thin, reusable DB entrypoint for application services.
///
/// This wraps a module-scoped `Db` and provides:
/// - `conn()` for non-transactional operations
/// - `transaction(...)` for transactional operations without exposing `DbHandle`
///
/// Services can store this behind an `Arc` and use:
///
/// ```ignore
/// let conn = self.db.conn()?;
/// let out = self.db.transaction(|tx| Box::pin(async move { /* ... */ })).await?;
/// ```
pub struct DBProvider<E> {
    db: Arc<Db>,
    _error: PhantomData<fn() -> E>,
}

impl<E> Clone for DBProvider<E> {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            _error: PhantomData,
        }
    }
}

impl<E> DBProvider<E>
where
    E: From<DbError> + Send + 'static,
{
    #[must_use]
    pub fn new(db: Db) -> Self {
        Self {
            db: Arc::new(db),
            _error: PhantomData,
        }
    }

    /// Create a non-transactional database runner.
    ///
    /// # Errors
    ///
    /// Returns `E` if `Db::conn()` fails (including the transaction-bypass guard).
    pub fn conn(&self) -> Result<DbConn<'_>, E> {
        self.db.conn().map_err(E::from)
    }

    /// Execute a closure inside a database transaction.
    ///
    /// # Errors
    ///
    /// Returns `E` if:
    /// - starting the transaction fails (mapped from `DbError`)
    /// - the closure returns an error
    /// - commit fails (mapped from `DbError`)
    pub async fn transaction<T, F>(&self, f: F) -> Result<T, E>
    where
        T: Send + 'static,
        F: for<'a> FnOnce(&'a DbTx<'a>) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>
            + Send,
    {
        self.db.transaction_ref_mapped(f).await
    }
}
