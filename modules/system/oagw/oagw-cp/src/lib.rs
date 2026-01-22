//! Data Plane implementation for OAGW.
//!
//! This crate implements the Data Plane component responsible for:
//! - Proxy orchestration and request execution
//! - Plugin execution (auth, guard, transform)
//! - HTTP calls to external upstream services
//!
//! Note: The crate name `oagw-cp` is retained for historical reasons.
//! This crate implements the Data Plane, not the Control Plane.

#![allow(dead_code)]

pub mod control_plane;
