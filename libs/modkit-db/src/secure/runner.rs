//! Hidden database runner capability.
//!
//! This module intentionally does **not** expose any raw `SeaORM` connection/transaction types
//! to downstream crates. It exists solely to allow secure query wrappers to execute queries
//! against either a normal connection (`SecureConn`) or an in-flight transaction (`SecureTx`).

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
