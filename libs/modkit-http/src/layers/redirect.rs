//! Secure redirect policy for HTTP clients
//!
//! This module provides a security-hardened redirect policy that protects against:
//! - SSRF (Server-Side Request Forgery) via cross-origin redirects
//! - Credential leakage via `Authorization` header forwarding
//! - HTTPS downgrade attacks
//!
//! ## Default Behavior
//!
//! By default, `SecureRedirectPolicy`:
//! - Only follows same-origin redirects (same scheme, host, and port)
//! - Strips sensitive headers (`Authorization`, `Cookie`, `Proxy-Authorization`) on cross-origin redirects
//! - Blocks HTTPS → HTTP downgrades
//! - Limits total redirects (configurable, default: 10)
//!
//! ## Configuration
//!
//! Use [`RedirectConfig`](crate::RedirectConfig) to customize behavior.

use crate::config::RedirectConfig;
use http::{Request, Uri, header};
use tower_http::follow_redirect::policy::{Action, Attempt, Policy};

/// Headers that are stripped on cross-origin redirects to prevent credential leakage
const SENSITIVE_HEADERS: &[header::HeaderName] = &[
    header::AUTHORIZATION,
    header::COOKIE,
    header::PROXY_AUTHORIZATION,
];

/// A security-hardened redirect policy
///
/// Implements [`tower_http::follow_redirect::policy::Policy`] with configurable
/// security controls.
///
/// ## Security Features
///
/// 1. **Same-origin enforcement**: By default, only follows redirects to the same host
/// 2. **Header stripping**: Removes `Authorization`, `Cookie` on cross-origin redirects
/// 3. **Downgrade protection**: Blocks HTTPS → HTTP redirects
/// 4. **Host allow-list**: Configurable list of trusted redirect targets
///
/// ## Example
///
/// ```rust,ignore
/// use modkit_http::{SecureRedirectPolicy, RedirectConfig};
///
/// let policy = SecureRedirectPolicy::new(RedirectConfig::default());
/// ```
#[derive(Debug, Clone)]
pub struct SecureRedirectPolicy {
    config: RedirectConfig,
    /// Track the number of redirects followed (resets per-request via Clone)
    redirect_count: usize,
    /// Track if we're in a cross-origin redirect chain (for header stripping)
    cross_origin_detected: bool,
}

impl SecureRedirectPolicy {
    /// Create a new secure redirect policy with the given configuration
    #[must_use]
    pub fn new(config: RedirectConfig) -> Self {
        Self {
            config,
            redirect_count: 0,
            cross_origin_detected: false,
        }
    }

    /// Check if the redirect is to the same origin (scheme, host, port)
    ///
    /// Missing schemes default to "https" (fail-closed): a scheme-less URI is
    /// treated as HTTPS so that cross-scheme comparisons err on the side of
    /// security rather than silently downgrading.
    fn is_same_origin(original: &Uri, target: &Uri) -> bool {
        let orig_scheme = original.scheme_str().unwrap_or("https");
        let target_scheme = target.scheme_str().unwrap_or("https");

        let orig_host = original.host().unwrap_or("");
        let target_host = target.host().unwrap_or("");

        let orig_port = original
            .port_u16()
            .unwrap_or_else(|| default_port(orig_scheme));
        let target_port = target
            .port_u16()
            .unwrap_or_else(|| default_port(target_scheme));

        orig_scheme == target_scheme && orig_host == target_host && orig_port == target_port
    }

    /// Check if the redirect is an HTTPS → HTTP downgrade
    fn is_https_downgrade(original: &Uri, target: &Uri) -> bool {
        let orig_scheme = original.scheme_str().unwrap_or("https");
        let target_scheme = target.scheme_str().unwrap_or("https");

        orig_scheme == "https" && target_scheme == "http"
    }

    /// Check if the target host is in the allowed hosts list
    fn is_allowed_host(&self, target: &Uri) -> bool {
        if let Some(host) = target.host() {
            self.config.allowed_redirect_hosts.contains(host)
        } else {
            false
        }
    }

    /// Determine if we should strip sensitive headers for this redirect
    #[cfg(test)]
    fn should_strip_headers(&self, original: &Uri, target: &Uri) -> bool {
        self.config.strip_sensitive_headers && !Self::is_same_origin(original, target)
    }
}

/// Get the default port for a scheme
fn default_port(scheme: &str) -> u16 {
    match scheme {
        "http" => 80,
        "https" => 443,
        _ => 0,
    }
}

impl<B: Clone, E> Policy<B, E> for SecureRedirectPolicy {
    fn redirect(&mut self, attempt: &Attempt<'_>) -> Result<Action, E> {
        // Check max redirects
        self.redirect_count += 1;
        if self.redirect_count > self.config.max_redirects {
            tracing::debug!(
                count = self.redirect_count,
                max = self.config.max_redirects,
                "Redirect limit reached"
            );
            return Ok(Action::Stop);
        }

        // previous() returns the original request URI
        let original = attempt.previous();
        let target = attempt.location();

        // Check HTTPS → HTTP downgrade
        if !self.config.allow_https_downgrade && Self::is_https_downgrade(original, target) {
            tracing::warn!(
                original = %original,
                target = %target,
                "Blocking HTTPS to HTTP downgrade redirect"
            );
            return Ok(Action::Stop);
        }

        // Check same-origin policy
        let is_same_origin = Self::is_same_origin(original, target);
        let is_allowed_host = self.is_allowed_host(target);

        if self.config.same_origin_only && !is_same_origin && !is_allowed_host {
            tracing::warn!(
                original = %original,
                target = %target,
                "Blocking cross-origin redirect (same_origin_only=true)"
            );
            return Ok(Action::Stop);
        }

        // Track if we've crossed origins for header stripping in on_request
        if !is_same_origin {
            self.cross_origin_detected = true;
            tracing::debug!(
                original = %original,
                target = %target,
                "Cross-origin redirect detected"
            );
        }

        Ok(Action::Follow)
    }

    fn on_request(&mut self, request: &mut Request<B>) {
        // Strip sensitive headers if we've detected a cross-origin redirect
        // This happens AFTER the redirect() decision, so we know we're following it
        if self.cross_origin_detected && self.config.strip_sensitive_headers {
            let headers = request.headers_mut();
            for header_name in SENSITIVE_HEADERS {
                if headers.remove(header_name).is_some() {
                    tracing::debug!(header = %header_name, "Stripped sensitive header on cross-origin redirect");
                }
            }
        }
    }

    fn clone_body(&self, body: &B) -> Option<B> {
        // Clone body for 307/308 redirects that require preserving the request body
        Some(body.clone())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn uri(s: &str) -> Uri {
        s.parse().unwrap()
    }

    #[test]
    fn test_is_same_origin_same() {
        assert!(SecureRedirectPolicy::is_same_origin(
            &uri("https://example.com/foo"),
            &uri("https://example.com/bar")
        ));
    }

    #[test]
    fn test_is_same_origin_different_host() {
        assert!(!SecureRedirectPolicy::is_same_origin(
            &uri("https://example.com/foo"),
            &uri("https://other.com/bar")
        ));
    }

    #[test]
    fn test_is_same_origin_different_scheme() {
        assert!(!SecureRedirectPolicy::is_same_origin(
            &uri("https://example.com/foo"),
            &uri("http://example.com/bar")
        ));
    }

    #[test]
    fn test_is_same_origin_different_port() {
        assert!(!SecureRedirectPolicy::is_same_origin(
            &uri("https://example.com/foo"),
            &uri("https://example.com:8443/bar")
        ));
    }

    #[test]
    fn test_is_same_origin_explicit_default_port() {
        // https with explicit 443 should match https without port
        assert!(SecureRedirectPolicy::is_same_origin(
            &uri("https://example.com/foo"),
            &uri("https://example.com:443/bar")
        ));
    }

    #[test]
    fn test_is_https_downgrade() {
        assert!(SecureRedirectPolicy::is_https_downgrade(
            &uri("https://example.com/foo"),
            &uri("http://example.com/bar")
        ));
    }

    #[test]
    fn test_is_not_https_downgrade() {
        // HTTP to HTTPS is an upgrade, not downgrade
        assert!(!SecureRedirectPolicy::is_https_downgrade(
            &uri("http://example.com/foo"),
            &uri("https://example.com/bar")
        ));

        // HTTPS to HTTPS is not a downgrade
        assert!(!SecureRedirectPolicy::is_https_downgrade(
            &uri("https://example.com/foo"),
            &uri("https://other.com/bar")
        ));
    }

    #[test]
    fn test_allowed_host() {
        let config = RedirectConfig {
            allowed_redirect_hosts: HashSet::from(["trusted.com".to_owned()]),
            ..Default::default()
        };
        let policy = SecureRedirectPolicy::new(config);

        assert!(policy.is_allowed_host(&uri("https://trusted.com/path")));
        assert!(!policy.is_allowed_host(&uri("https://untrusted.com/path")));
    }

    #[test]
    fn test_redirect_config_default() {
        let config = RedirectConfig::default();
        assert_eq!(config.max_redirects, 10);
        assert!(config.same_origin_only);
        assert!(config.strip_sensitive_headers);
        assert!(!config.allow_https_downgrade);
        assert!(config.allowed_redirect_hosts.is_empty());
    }

    #[test]
    fn test_redirect_config_permissive() {
        let config = RedirectConfig::permissive();
        assert_eq!(config.max_redirects, 10);
        assert!(!config.same_origin_only);
        assert!(config.strip_sensitive_headers);
        assert!(!config.allow_https_downgrade);
    }

    #[test]
    fn test_redirect_config_disabled() {
        let config = RedirectConfig::disabled();
        assert_eq!(config.max_redirects, 0);
    }

    #[test]
    fn test_redirect_config_for_testing() {
        let config = RedirectConfig::for_testing();
        assert!(!config.same_origin_only);
        assert!(config.allow_https_downgrade);
        assert!(config.strip_sensitive_headers); // Still strip headers
    }

    #[test]
    fn test_should_strip_headers() {
        let config = RedirectConfig::default();
        let policy = SecureRedirectPolicy::new(config);

        // Same origin - don't strip
        assert!(
            !policy
                .should_strip_headers(&uri("https://example.com/a"), &uri("https://example.com/b"))
        );

        // Cross origin - strip
        assert!(
            policy.should_strip_headers(&uri("https://example.com/a"), &uri("https://other.com/b"))
        );
    }

    #[test]
    fn test_should_strip_headers_disabled() {
        let config = RedirectConfig {
            strip_sensitive_headers: false,
            ..Default::default()
        };
        let policy = SecureRedirectPolicy::new(config);

        // Cross origin but stripping disabled
        assert!(
            !policy
                .should_strip_headers(&uri("https://example.com/a"), &uri("https://other.com/b"))
        );
    }

    #[test]
    fn test_policy_new() {
        let config = RedirectConfig::default();
        let policy = SecureRedirectPolicy::new(config);
        assert_eq!(policy.redirect_count, 0);
        assert!(!policy.cross_origin_detected);
    }
}
