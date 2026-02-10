use serde::Deserialize;

pub use modkit_utils::SecretString;

/// `OAuth2` client authentication method.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ClientAuthMethod {
    /// HTTP Basic authentication (RFC 6749 §2.3.1).
    /// `Authorization: Basic base64(client_id:client_secret)`
    #[default]
    Basic,
    /// Credentials in the request body (RFC 6749 §2.3.1 alternative).
    /// `client_id` and `client_secret` as form fields.
    Form,
}

/// Deserialized `OAuth2` token endpoint response.
///
/// Only the fields required by the client credentials flow are included.
/// Unknown fields are silently ignored during deserialization.
///
/// **Intentionally `Deserialize`-only** — `Serialize` is not derived to
/// prevent accidental serialization of access tokens into logs or
/// error messages.
#[derive(Deserialize)]
pub(crate) struct TokenResponse {
    /// The access token issued by the authorization server.
    pub access_token: String,
    /// The lifetime in seconds of the access token (optional per RFC 6749).
    #[serde(default)]
    pub expires_in: Option<u64>,
    /// The type of the token issued (optional; must be "Bearer" if present).
    #[serde(default)]
    pub token_type: Option<String>,
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn default_auth_method_is_basic() {
        assert_eq!(ClientAuthMethod::default(), ClientAuthMethod::Basic);
    }

    #[test]
    fn deserialize_full_response() {
        let json = r#"{"access_token":"tok","expires_in":3600,"token_type":"Bearer"}"#;
        let r: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.access_token, "tok");
        assert_eq!(r.expires_in, Some(3600));
        assert_eq!(r.token_type.as_deref(), Some("Bearer"));
    }

    #[test]
    fn deserialize_minimal_response() {
        let json = r#"{"access_token":"tok"}"#;
        let r: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.access_token, "tok");
        assert!(r.expires_in.is_none());
        assert!(r.token_type.is_none());
    }

    #[test]
    fn deserialize_ignores_unknown_fields() {
        let json = r#"{"access_token":"tok","scope":"read","refresh_token":"rt"}"#;
        let r: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.access_token, "tok");
    }
}
