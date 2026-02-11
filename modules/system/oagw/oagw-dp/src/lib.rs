//! Control Plane implementation for OAGW.
//!
//! This crate implements the Control Plane component responsible for:
//! - Configuration management (CRUD for upstreams, routes, plugins)
//! - Database access and persistence
//! - Multi-layer caching (L1 in-memory + L2 Redis)
//!
//! Note: The crate name `oagw-dp` is retained for historical reasons.
//! This crate implements the Control Plane, not the Data Plane.

#![allow(dead_code)]

pub mod data_plane;
