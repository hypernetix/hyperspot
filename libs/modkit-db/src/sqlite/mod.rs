//! SQLite-specific helpers and utilities.
//!
//! This module contains SQLite-specific functionality including:
//! - DSN parsing and cleaning
//! - PRAGMA parameter handling with typed enums
//! - Path preparation for `SQLite` databases

pub(crate) mod dsn;
pub(crate) mod path;
pub(crate) mod pragmas;

pub(crate) use dsn::{extract_sqlite_pragmas, is_memory_dsn};
pub(crate) use path::prepare_sqlite_path;
pub(crate) use pragmas::Pragmas;
