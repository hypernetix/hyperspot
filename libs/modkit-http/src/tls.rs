//! TLS utilities for the HTTP client.
//!
//! This module provides cached loading of native root certificates to avoid
//! repeated OS certificate store lookups (which can be slow on some platforms).

use rustls_pki_types::CertificateDer;
use std::sync::{Arc, OnceLock};

/// Cached native root certificates.
/// Always stores Ok; empty vec means no certs found (warned, not errored).
static NATIVE_ROOTS_CACHE: OnceLock<Vec<CertificateDer<'static>>> = OnceLock::new();

/// Counter for test verification that the loader only runs once.
#[cfg(test)]
static LOAD_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Load native root certificates from the OS certificate store.
///
/// This function is called once and the result is cached for subsequent calls.
/// Returns Ok with potentially empty vec; missing certs are warned, not errored.
fn load_native_certs_inner() -> Vec<CertificateDer<'static>> {
    #[cfg(test)]
    LOAD_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let result = rustls_native_certs::load_native_certs();

    // Log any errors encountered during loading
    if !result.errors.is_empty() {
        for err in &result.errors {
            tracing::warn!(error = %err, "error loading native root certificate");
        }
    }

    let certs: Vec<CertificateDer<'static>> = result.certs;

    if certs.is_empty() {
        tracing::warn!("no native root CA certificates found");
    } else {
        tracing::debug!(count = certs.len(), "loaded native root certificates");
    }

    certs
}

/// Get cached native root certificates.
///
/// Returns a reference to the cached certificates (may be empty).
/// The certificates are loaded lazily on first call and cached for all subsequent calls.
pub fn native_root_certs() -> &'static [CertificateDer<'static>] {
    NATIVE_ROOTS_CACHE
        .get_or_init(load_native_certs_inner)
        .as_slice()
}

/// Get the crypto provider for TLS connections.
///
/// This function follows the reqwest pattern:
/// 1. Check if a default provider is already installed globally
/// 2. If yes, use that (respects user configuration)
/// 3. If no, create a new aws-lc-rs provider without installing it globally
///
/// This avoids global state mutation and is safe to call from multiple threads.
pub fn get_crypto_provider() -> Arc<rustls::crypto::CryptoProvider> {
    rustls::crypto::CryptoProvider::get_default()
        .cloned()
        .unwrap_or_else(|| Arc::new(rustls::crypto::aws_lc_rs::default_provider()))
}

/// Build a rustls `ClientConfig` using the cached native root certificates.
///
/// # Errors
///
/// Returns an error if no valid root certificates are available:
/// - OS certificate store is empty
/// - All certificates failed to parse
///
/// This fail-fast behavior ensures TLS configuration errors are caught at client
/// construction time rather than failing later during TLS handshakes.
pub fn native_roots_client_config() -> Result<rustls::ClientConfig, String> {
    let certs = native_root_certs();

    let mut root_store = rustls::RootCertStore::empty();

    if certs.is_empty() {
        return Err("no native root CA certificates found in OS certificate store".to_owned());
    }

    let (added, ignored) = root_store.add_parsable_certificates(certs.iter().cloned());

    if ignored > 0 {
        tracing::warn!(
            added = added,
            ignored = ignored,
            "some native root certificates could not be parsed"
        );
    }

    if added == 0 {
        return Err(format!(
            "no valid native root CA certificates parsed (found {}, all {} failed to parse)",
            certs.len(),
            ignored
        ));
    }

    let provider = get_crypto_provider();

    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("failed to set TLS protocol versions: {e}"))?
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(config)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    /// Test that native root certs are cached after the first load.
    ///
    /// NOTE: This test verifies "at most one load" rather than "exactly one load"
    /// because `LOAD_COUNT` is a global atomic shared across all tests. If another
    /// test (or parallel test) calls `native_root_certs()` before this test runs,
    /// the cache will already be initialized and `final_count - initial_count`
    /// will be 0. The assertion handles this correctly.
    #[test]
    fn test_native_roots_cached() {
        // Capture count before our calls (may be non-zero if cache already initialized)
        let initial_count = LOAD_COUNT.load(Ordering::SeqCst);

        // First call - loads if not cached, otherwise uses existing cache
        let result1 = native_root_certs();

        // Second call should use cache
        let result2 = native_root_certs();

        // Third call should also use cache
        let result3 = native_root_certs();

        // Verify loader was called at most once more than initial (0 if already cached, 1 if we triggered the load)
        let final_count = LOAD_COUNT.load(Ordering::SeqCst);
        assert!(
            final_count <= initial_count + 1,
            "loader should run at most once, but ran {} times since test start",
            final_count - initial_count
        );

        // Results should be consistent (same slice pointer)
        assert_eq!(result1.len(), result2.len());
        assert_eq!(result2.len(), result3.len());
        assert!(std::ptr::eq(result1, result2), "should return same slice");
        assert!(std::ptr::eq(result2, result3), "should return same slice");
    }

    #[test]
    fn test_native_roots_client_config() {
        // Building client config succeeds if native roots are available
        // (which they should be on most CI/dev systems)
        // On systems without native certs, this returns Err (expected behavior)
        let result = native_roots_client_config();

        // Log the result for debugging in CI
        match &result {
            Ok(_) => tracing::debug!("native_roots_client_config succeeded"),
            Err(e) => {
                tracing::debug!(error = %e, "native_roots_client_config failed (expected on minimal containers)");
            }
        }

        // We don't assert success because CI containers may not have OS certs.
        // The important thing is it doesn't panic.
    }
}
