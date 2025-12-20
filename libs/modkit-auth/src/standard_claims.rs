//! Standard JWT claim names as defined in RFC 7519.
//!
//! This module provides type-safe constants for standard JWT claim names,
//! reducing the risk of typos and providing a central place for claim name definitions.
//!
//! # References
//! - [RFC 7519 - JSON Web Token (JWT)](https://datatracker.ietf.org/doc/html/rfc7519)
//! - [IANA JWT Claims Registry](https://www.iana.org/assignments/jwt/jwt.xhtml)

/// Standard JWT claim names as defined in RFC 7519 and OIDC specifications.
///
/// This struct provides constants for standard claim names used in JWT tokens.
/// Using these constants instead of string literals helps prevent typos and
/// provides a single source of truth for claim names.
///
/// # Example
/// ```
/// use modkit_auth::StandardClaim;
/// use serde_json::json;
///
/// let claims = json!({
///     "sub": "user-123",
///     "iss": "https://auth.example.com"
/// });
///
/// let subject = claims.get(StandardClaim::SUB);
/// let issuer = claims.get(StandardClaim::ISS);
/// ```
pub struct StandardClaim;

impl StandardClaim {
    // =========================================================================
    // RFC 7519 Registered Claims (Section 4.1)
    // =========================================================================

    /// Issuer claim - identifies the principal that issued the JWT.
    ///
    /// The "iss" (issuer) claim identifies the principal that issued the JWT.
    /// The processing of this claim is generally application specific.
    ///
    /// See: <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.1>
    pub const ISS: &'static str = "iss";

    /// Subject claim - identifies the principal that is the subject of the JWT.
    ///
    /// The "sub" (subject) claim identifies the principal that is the subject
    /// of the JWT. The claims in a JWT are normally statements about the subject.
    ///
    /// See: <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.2>
    pub const SUB: &'static str = "sub";

    /// Audience claim - identifies the recipients that the JWT is intended for.
    ///
    /// The "aud" (audience) claim identifies the recipients that the JWT is
    /// intended for. Each principal intended to process the JWT MUST identify
    /// itself with a value in the audience claim.
    ///
    /// See: <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.3>
    pub const AUD: &'static str = "aud";

    /// Expiration Time claim - identifies the expiration time of the JWT.
    ///
    /// The "exp" (expiration time) claim identifies the expiration time on
    /// or after which the JWT MUST NOT be accepted for processing.
    ///
    /// See: <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.4>
    pub const EXP: &'static str = "exp";

    /// Not Before claim - identifies the time before which the JWT must not be accepted.
    ///
    /// The "nbf" (not before) claim identifies the time before which the JWT
    /// MUST NOT be accepted for processing.
    ///
    /// See: <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.5>
    pub const NBF: &'static str = "nbf";

    /// Issued At claim - identifies the time at which the JWT was issued.
    ///
    /// The "iat" (issued at) claim identifies the time at which the JWT was issued.
    /// This claim can be used to determine the age of the JWT.
    ///
    /// See: <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.6>
    pub const IAT: &'static str = "iat";

    /// JWT ID claim - provides a unique identifier for the JWT.
    ///
    /// The "jti" (JWT ID) claim provides a unique identifier for the JWT.
    /// The identifier value MUST be assigned in a manner that ensures that
    /// there is a negligible probability that the same value will be
    /// accidentally assigned to a different data object.
    ///
    /// See: <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.7>
    pub const JTI: &'static str = "jti";

    // =========================================================================
    // OpenID Connect Claims
    // =========================================================================

    /// Authorized party claim - the party to which the ID Token was issued.
    ///
    /// The "azp" (authorized party) claim identifies the party to which the
    /// ID Token was issued. If present, it MUST contain the OAuth 2.0 Client ID
    /// of this party.
    ///
    /// See: <https://openid.net/specs/openid-connect-core-1_0.html#IDToken>
    pub const AZP: &'static str = "azp";

    /// Returns a slice containing all standard JWT claim names (RFC 7519).
    ///
    /// This is useful for filtering out standard claims when collecting
    /// extra/custom claims from a token.
    #[must_use]
    pub const fn all_registered() -> &'static [&'static str] {
        &[
            Self::ISS,
            Self::SUB,
            Self::AUD,
            Self::EXP,
            Self::NBF,
            Self::IAT,
            Self::JTI,
            Self::AZP,
        ]
    }

    /// Checks if the given claim name is a standard registered claim.
    #[must_use]
    pub fn is_registered(name: &str) -> bool {
        Self::all_registered().contains(&name)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_claim_constants() {
        assert_eq!(StandardClaim::ISS, "iss");
        assert_eq!(StandardClaim::SUB, "sub");
        assert_eq!(StandardClaim::AUD, "aud");
        assert_eq!(StandardClaim::EXP, "exp");
        assert_eq!(StandardClaim::NBF, "nbf");
        assert_eq!(StandardClaim::IAT, "iat");
        assert_eq!(StandardClaim::JTI, "jti");
        assert_eq!(StandardClaim::AZP, "azp");
    }

    #[test]
    fn test_all_registered() {
        let all = StandardClaim::all_registered();
        assert_eq!(all.len(), 8);
        assert!(all.contains(&"iss"));
        assert!(all.contains(&"sub"));
        assert!(all.contains(&"aud"));
        assert!(all.contains(&"exp"));
        assert!(all.contains(&"nbf"));
        assert!(all.contains(&"iat"));
        assert!(all.contains(&"jti"));
        assert!(all.contains(&"azp"));
    }

    #[test]
    fn test_is_registered() {
        assert!(StandardClaim::is_registered("iss"));
        assert!(StandardClaim::is_registered("sub"));
        assert!(StandardClaim::is_registered("azp"));
        assert!(!StandardClaim::is_registered("custom_claim"));
        assert!(!StandardClaim::is_registered("tenant_id"));
        assert!(!StandardClaim::is_registered("roles"));
    }
}
