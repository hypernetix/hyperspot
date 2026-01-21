//! Transaction error types for typed domain transactions.
//!
//! These types allow domain errors to be propagated through `SeaORM` transactions
//! without mutex-based storage or string parsing.

use std::fmt;

/// Infrastructure error representing a database-level failure.
///
/// This wraps database errors (connection issues, constraint violations, etc.)
/// in a type that does not expose `SeaORM` internals.
#[derive(Debug, Clone)]
pub struct InfraError {
    message: String,
}

impl InfraError {
    /// Create a new infrastructure error from a message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Get the error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for InfraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for InfraError {}

/// Transaction error that distinguishes domain errors from infrastructure errors.
///
/// This type is returned by [`SecureConn::in_transaction`] and allows callers to
/// handle domain errors separately from database infrastructure failures.
///
/// # Example
///
/// ```ignore
/// use modkit_db::secure::{SecureConn, TxError};
///
/// let result = db.in_transaction(|tx| Box::pin(async move {
///     // Domain logic that may return DomainError
///     repo.create(tx, user).await
/// })).await;
///
/// let user = result.map_err(|e| e.into_domain(DomainError::database_infra))?;
/// ```
#[derive(Debug, Clone)]
pub enum TxError<E> {
    /// A domain error returned from the transaction callback.
    Domain(E),
    /// An infrastructure error from the database layer.
    Infra(InfraError),
}

impl<E> TxError<E> {
    /// Convert this transaction error into a domain error.
    ///
    /// If this is already a domain error, returns it directly.
    /// If this is an infrastructure error, uses the provided mapping function
    /// to convert it into a domain error.
    pub fn into_domain<F>(self, map_infra: F) -> E
    where
        F: FnOnce(InfraError) -> E,
    {
        match self {
            TxError::Domain(e) => e,
            TxError::Infra(infra) => map_infra(infra),
        }
    }
}

impl<E: fmt::Display> fmt::Display for TxError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxError::Domain(e) => write!(f, "{e}"),
            TxError::Infra(e) => write!(f, "infrastructure error: {e}"),
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for TxError<E> {}
