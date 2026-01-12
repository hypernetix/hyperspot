//! Transaction configuration types for `SecureConn`.
//!
//! These types abstract `SeaORM`'s transaction configuration, allowing domain
//! and application services to specify transaction settings without importing
//! `SeaORM` types directly.
//!
//! # Design Philosophy
//!
//! - **Isolation**: These types are exposed from `modkit_db` but do NOT expose
//!   any `SeaORM` types. The conversion to `SeaORM` types happens internally.
//! - **REST handlers** should never use these types directly. Transaction
//!   boundaries belong in application/domain services.
//! - **Domain services** may use `TxConfig` to specify transaction requirements.
//!
//! # Example
//!
//! ```ignore
//! use modkit_db::secure::{SecureConn, TxConfig, TxIsolationLevel, TxAccessMode};
//!
//! // In a domain service:
//! pub async fn transfer_funds(
//!     db: &SecureConn,
//!     from: Uuid,
//!     to: Uuid,
//!     amount: Decimal,
//! ) -> anyhow::Result<()> {
//!     let cfg = TxConfig {
//!         isolation: Some(TxIsolationLevel::Serializable),
//!         access_mode: Some(TxAccessMode::ReadWrite),
//!     };
//!
//!     db.transaction_with_config(cfg, |tx| async move {
//!         accounts_repo.debit(from, amount, tx).await?;
//!         accounts_repo.credit(to, amount, tx).await?;
//!         Ok(())
//!     }).await
//! }
//! ```

/// Transaction isolation level.
///
/// Controls how transaction integrity is maintained when multiple transactions
/// access the same data concurrently.
///
/// # Variants
///
/// - `ReadUncommitted`: Lowest isolation. Allows dirty reads.
/// - `ReadCommitted`: Prevents dirty reads. Default for most databases.
/// - `RepeatableRead`: Prevents dirty reads and non-repeatable reads.
/// - `Serializable`: Highest isolation. Transactions are fully serialized.
///
/// # Backend Notes
///
/// - **`PostgreSQL`**: Supports all levels. `RepeatableRead` actually uses
///   snapshot isolation.
/// - **MySQL/InnoDB**: Supports all levels.
/// - **`SQLite`**: Only supports `Serializable` (the default). Other levels
///   are mapped to `Serializable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TxIsolationLevel {
    /// Allows dirty reads. Not recommended for most use cases.
    ReadUncommitted,
    /// Prevents dirty reads. This is the default for most databases.
    #[default]
    ReadCommitted,
    /// Prevents dirty reads and non-repeatable reads.
    RepeatableRead,
    /// Full serialization of transactions. Highest isolation level.
    Serializable,
}

/// Transaction access mode.
///
/// Specifies whether the transaction will modify data or only read it.
///
/// # Variants
///
/// - `ReadOnly`: Transaction will not modify data. May enable optimizations.
/// - `ReadWrite`: Transaction may modify data (default).
///
/// # Backend Notes
///
/// - **`PostgreSQL`**: `READ ONLY` transactions reject any write operations.
/// - **`MySQL`**: Supports `READ ONLY` mode for `InnoDB`.
/// - **`SQLite`**: Read-only mode is not explicitly supported; this is a hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TxAccessMode {
    /// Transaction will only read data.
    ReadOnly,
    /// Transaction may read and write data (default).
    #[default]
    ReadWrite,
}

/// Configuration for database transactions.
///
/// Use this struct to specify transaction isolation level and access mode
/// without importing `SeaORM` types.
///
/// # Example
///
/// ```ignore
/// use modkit_db::secure::{TxConfig, TxIsolationLevel, TxAccessMode};
///
/// // Default configuration (database defaults)
/// let default_cfg = TxConfig::default();
///
/// // Explicit configuration
/// let cfg = TxConfig {
///     isolation: Some(TxIsolationLevel::RepeatableRead),
///     access_mode: Some(TxAccessMode::ReadOnly),
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct TxConfig {
    /// Transaction isolation level. If `None`, uses database default.
    pub isolation: Option<TxIsolationLevel>,
    /// Transaction access mode. If `None`, uses database default (usually `ReadWrite`).
    pub access_mode: Option<TxAccessMode>,
}

impl TxConfig {
    /// Create a new configuration with the specified isolation level.
    #[must_use]
    pub fn with_isolation(isolation: TxIsolationLevel) -> Self {
        Self {
            isolation: Some(isolation),
            access_mode: None,
        }
    }

    /// Create a read-only transaction configuration.
    #[must_use]
    pub fn read_only() -> Self {
        Self {
            isolation: None,
            access_mode: Some(TxAccessMode::ReadOnly),
        }
    }

    /// Create a serializable transaction configuration.
    ///
    /// This is the highest isolation level, ensuring full serialization
    /// of transactions.
    #[must_use]
    pub fn serializable() -> Self {
        Self {
            isolation: Some(TxIsolationLevel::Serializable),
            access_mode: None,
        }
    }
}

// ============================================================================
// SeaORM conversions (internal to modkit-db)
// ============================================================================

use sea_orm::{AccessMode, IsolationLevel};

impl From<TxIsolationLevel> for IsolationLevel {
    fn from(level: TxIsolationLevel) -> Self {
        match level {
            TxIsolationLevel::ReadUncommitted => IsolationLevel::ReadUncommitted,
            TxIsolationLevel::ReadCommitted => IsolationLevel::ReadCommitted,
            TxIsolationLevel::RepeatableRead => IsolationLevel::RepeatableRead,
            TxIsolationLevel::Serializable => IsolationLevel::Serializable,
        }
    }
}

impl From<TxAccessMode> for AccessMode {
    fn from(mode: TxAccessMode) -> Self {
        match mode {
            TxAccessMode::ReadOnly => AccessMode::ReadOnly,
            TxAccessMode::ReadWrite => AccessMode::ReadWrite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tx_config() {
        let cfg = TxConfig::default();
        assert!(cfg.isolation.is_none());
        assert!(cfg.access_mode.is_none());
    }

    #[test]
    fn test_tx_config_with_isolation() {
        let cfg = TxConfig::with_isolation(TxIsolationLevel::Serializable);
        assert_eq!(cfg.isolation, Some(TxIsolationLevel::Serializable));
        assert!(cfg.access_mode.is_none());
    }

    #[test]
    fn test_tx_config_read_only() {
        let cfg = TxConfig::read_only();
        assert!(cfg.isolation.is_none());
        assert_eq!(cfg.access_mode, Some(TxAccessMode::ReadOnly));
    }

    #[test]
    fn test_tx_config_serializable() {
        let cfg = TxConfig::serializable();
        assert_eq!(cfg.isolation, Some(TxIsolationLevel::Serializable));
        assert!(cfg.access_mode.is_none());
    }

    #[test]
    fn test_isolation_level_conversion() {
        assert!(matches!(
            IsolationLevel::from(TxIsolationLevel::ReadUncommitted),
            IsolationLevel::ReadUncommitted
        ));
        assert!(matches!(
            IsolationLevel::from(TxIsolationLevel::ReadCommitted),
            IsolationLevel::ReadCommitted
        ));
        assert!(matches!(
            IsolationLevel::from(TxIsolationLevel::RepeatableRead),
            IsolationLevel::RepeatableRead
        ));
        assert!(matches!(
            IsolationLevel::from(TxIsolationLevel::Serializable),
            IsolationLevel::Serializable
        ));
    }

    #[test]
    fn test_access_mode_conversion() {
        assert!(matches!(
            AccessMode::from(TxAccessMode::ReadOnly),
            AccessMode::ReadOnly
        ));
        assert!(matches!(
            AccessMode::from(TxAccessMode::ReadWrite),
            AccessMode::ReadWrite
        ));
    }
}
