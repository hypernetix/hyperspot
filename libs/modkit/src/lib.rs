//! # ModKit - Declarative Module System
//!
//! A unified crate for building modular applications with declarative module definitions.
//!
//! ## Features
//!
//! - **Declarative**: Use `#[module(...)]` attribute to declare modules
//! - **Auto-discovery**: Modules are automatically discovered via inventory
//! - **Type-safe**: Compile-time validation of capabilities
//! - **Phase-based lifecycle**: DB → init → REST → start → stop
//!
//! ## Example
//!
//! ```rust,ignore
//! use modkit::{module, Module, DbModule, RestfulModule, StatefulModule};
//!
//! #[derive(Default)]
//! #[module(name = "user", deps = ["database"], caps = [db, rest, stateful])]
//! pub struct UserModule;
//!
//! // Implement the declared capabilities...
//! ```

pub use anyhow::Result;
pub use async_trait::async_trait;

// Re-export inventory for user convenience
pub use inventory;

// Module system exports
pub use crate::contracts::*;
pub mod context;
pub use context::{ConfigProvider, ModuleCtx, ModuleCtxBuilder};

// Module system implementations for macro code
pub mod client_hub;
pub mod registry;

// Re-export main types
pub use client_hub::ClientHub;
pub use registry::ModuleRegistry;

// Re-export the macros from the proc-macro crate
pub use modkit_macros::{lifecycle, module};

// Core module contracts and traits
pub mod contracts;
// Type-safe API operation builder
pub mod api;
pub use api::{OpenApiRegistry, OperationBuilder};

// HTTP utilities
pub mod http;
pub use http::problem::{
    bad_request, conflict, internal_error, not_found, Problem, ProblemResponse, ValidationError,
};

pub mod lifecycle;
pub mod runtime;

pub use lifecycle::{Lifecycle, Runnable, Status, StopReason, WithLifecycle};
pub use runtime::{run, DbOptions, RunOptions, ShutdownOptions};

#[cfg(test)]
mod tests;
