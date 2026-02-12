//! Build metadata types and build-script helper for binary crates.
//!
//! # Binary crates (full setup)
//!
//! Binary crates that want to log build metadata on startup need **three**
//! pieces:
//!
//! 1. A **build-dependency** with the `shadow-rs` feature so `emit()` is
//!    available in `build.rs`:
//!
//!    ```toml
//!    [build-dependencies]
//!    cf-modkit-build = { workspace = true, features = ["shadow-rs"] }
//!    ```
//!
//! 2. A **runtime dependency** (types + macro only, no heavy deps):
//!
//!    ```toml
//!    [dependencies]
//!    cf-modkit-build = { workspace = true }
//!    ```
//!
//! 3. A `build.rs` that calls [`emit`]:
//!
//!    ```rust,ignore
//!    fn main() -> Result<(), Box<dyn std::error::Error>> {
//!        cf_modkit_build::emit()
//!    }
//!    ```
//!
//! Then in `main.rs`:
//!
//! ```rust,ignore
//! shadow_rs::shadow!(shadow);
//!
//! fn main() {
//!     // ...initialise logging...
//!     log_build_metadata(&cf_modkit_build::build_metadata!());
//! }
//! ```

// ── Runtime types (always available) ────────────────────────────────────

/// Build-time metadata collected by `shadow-rs` in each binary crate.
#[derive(Debug, Clone)]
pub struct BuildMetadata<'a> {
    /// Package version from `Cargo.toml` (e.g. `"0.2.8"`).
    pub version: &'a str,
    /// Full git commit hash.
    pub commit_hash: &'a str,
    /// Git branch name.
    pub branch: &'a str,
    /// Build timestamp (RFC 3339).
    pub build_time: &'a str,
    /// Rust compiler version string.
    pub rust_version: &'a str,
    /// Compilation target triple (e.g. `x86_64-unknown-linux-gnu`).
    pub build_target: &'a str,
    /// Comma-separated list of enabled Cargo feature flags.
    pub features: &'a str,
}

/// Log build metadata at `INFO` level.
///
/// Call this once after the tracing subscriber has been installed.
pub fn log_build_metadata(meta: &BuildMetadata<'_>) {
    let features = if meta.features.is_empty() {
        "(none)"
    } else {
        meta.features
    };

    tracing::info!(
        version = %meta.version,
        commit = %meta.commit_hash,
        branch = %meta.branch,
        build_time = %meta.build_time,
        rust = %meta.rust_version,
        target = %meta.build_target,
        features = %features,
        "Build metadata"
    );
}

/// Construct a [`BuildMetadata`] from the `shadow!(shadow)` module constants
/// and the `ENABLED_FEATURES` / `BUILD_TARGET` env vars emitted by [`emit`].
///
/// # Prerequisites
///
/// - The crate's `build.rs` must call [`emit`] (sets `ENABLED_FEATURES` and
///   `BUILD_TARGET` env vars).
/// - `shadow_rs::shadow!(shadow);` must appear at module scope in the caller.
#[macro_export]
macro_rules! build_metadata {
    () => {
        $crate::BuildMetadata {
            version: shadow::PKG_VERSION,
            commit_hash: shadow::COMMIT_HASH,
            branch: shadow::BRANCH,
            build_time: shadow::BUILD_TIME_3339,
            rust_version: shadow::RUST_VERSION,
            build_target: env!("BUILD_TARGET"),
            features: env!("ENABLED_FEATURES"),
        }
    };
}

// ── Build-script helper (requires `shadow-rs` feature) ─────────────────

/// Run shadow-rs and emit extra compile-time environment variables.
///
/// After this call the binary crate has access to:
/// - All standard `shadow-rs` constants via `shadow_rs::shadow!(shadow);`
/// - `env!("ENABLED_FEATURES")` — comma-separated list of enabled Cargo features
/// - `env!("BUILD_TARGET")` — full target triple (e.g. `x86_64-unknown-linux-gnu`)
///
/// # Errors
///
/// Returns an error if shadow-rs initialisation fails.
#[cfg(feature = "shadow-rs")]
#[allow(unknown_lints, de1301_no_print_macros)] // for special usage in build.rs
pub fn emit() -> Result<(), Box<dyn std::error::Error>> {
    shadow_rs::ShadowBuilder::builder().build()?;

    // Collect enabled Cargo features into a compile-time env var.
    let mut features: Vec<String> = std::env::vars()
        .filter_map(|(key, _)| {
            key.strip_prefix("CARGO_FEATURE_")
                .map(|f| f.to_lowercase().replace('_', "-"))
        })
        .collect();
    features.sort();
    println!("cargo:rustc-env=ENABLED_FEATURES={}", features.join(", "));

    // Expose the full target triple (e.g. x86_64-unknown-linux-gnu).
    if let Ok(target) = std::env::var("TARGET") {
        println!("cargo:rustc-env=BUILD_TARGET={target}");
    }

    Ok(())
}
