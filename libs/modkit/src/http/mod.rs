//! HTTP utilities for modkit
//!
//! This module provides shared HTTP types and utilities for building
//! modular web applications.

pub mod sse;

/// Conditionally enables insecure HTTP on an `HttpClient` builder in debug builds.
#[macro_export]
macro_rules! maybe_allow_insecure_http {
    ($builder:expr, $enabled:expr) => {{
        let builder = $builder;
        if $enabled {
            #[cfg(debug_assertions)]
            {
                builder.allow_insecure_http()
            }

            #[cfg(not(debug_assertions))]
            {
                builder
            }
        } else {
            builder
        }
    }};
}
