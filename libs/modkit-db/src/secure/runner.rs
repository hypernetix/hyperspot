//! Hidden database runner capability.
//!
//! This module intentionally does **not** expose any raw `SeaORM` connection/transaction types
//! to downstream crates. It exists solely to allow secure query wrappers to execute queries
//! against either a normal connection (`DbConn`) or an in-flight transaction (`DbTx`).
//!
//! # Security Model
//!
//! The `DBRunner` trait is **sealed** - it cannot be implemented outside this crate.
//! This ensures that only `DbConn` and `DbTx` can be used as database runners,
//! preventing user code from creating custom runners that could bypass transaction isolation.

use super::db::{DbConn, DbTx};
use super::secure_conn::{SecureConn, SecureTx};

mod sealed {
    pub trait Sealed {}
}

/// Internal-only bridge to `SeaORM`'s executor trait.
///
/// Downstream crates must never see or name `ConnectionTrait`, `DatabaseConnection`, or
/// `DatabaseTransaction`. This bridge is crate-only.
pub enum SeaOrmRunner<'a> {
    Conn(&'a sea_orm::DatabaseConnection),
    Tx(&'a sea_orm::DatabaseTransaction),
}

/// Internal-only bridge to `SeaORM`'s executor types.
pub trait DBRunnerInternal: sealed::Sealed + Send + Sync {
    fn as_seaorm(&self) -> SeaOrmRunner<'_>;
}

/// Hidden capability marker used by repositories and services.
///
/// This trait intentionally has **no methods** and cannot be implemented outside `modkit-db`.
///
/// Note: while `DBRunner` extends an internal trait, downstream crates cannot name that
/// internal trait, and therefore cannot obtain any raw SeaORM executor from a `DBRunner`.
#[doc(hidden)]
pub trait DBRunner: DBRunnerInternal {}

// --- New secure types (DbConn, DbTx) ---

impl sealed::Sealed for DbConn<'_> {}
impl DBRunnerInternal for DbConn<'_> {
    fn as_seaorm(&self) -> SeaOrmRunner<'_> {
        SeaOrmRunner::Conn(self.conn)
    }
}
impl DBRunner for DbConn<'_> {}

impl sealed::Sealed for DbTx<'_> {}
impl DBRunnerInternal for DbTx<'_> {
    fn as_seaorm(&self) -> SeaOrmRunner<'_> {
        SeaOrmRunner::Tx(self.tx)
    }
}
impl DBRunner for DbTx<'_> {}

// --- Legacy types (SecureConn, SecureTx) - kept for migration period ---

impl sealed::Sealed for SecureConn {}
impl DBRunnerInternal for SecureConn {
    fn as_seaorm(&self) -> SeaOrmRunner<'_> {
        SeaOrmRunner::Conn(&self.conn)
    }
}
impl DBRunner for SecureConn {}

impl sealed::Sealed for SecureTx<'_> {}
impl DBRunnerInternal for SecureTx<'_> {
    fn as_seaorm(&self) -> SeaOrmRunner<'_> {
        SeaOrmRunner::Tx(self.tx)
    }
}
impl DBRunner for SecureTx<'_> {}
