use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

pub struct RegisteredClaims {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, Eq, PartialOrd)]
pub struct Permission {
    resource: String,
    action: String,
}

impl Permission {
    pub fn new(resource: &str, action: &str) -> Self {
        Self {
            resource: resource.to_string(),
            action: action.to_string(),
        }
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn matches(&self, other: &Permission) -> bool {
        // TODO: use GTS library to check wildcards more robustly
        (self.resource == other.resource || self.resource == "*" || other.resource == "*")
            && (self.action == other.action || self.action == "*" || other.action == "*")
    }
}

impl From<&str> for Permission {
    fn from(role_str: &str) -> Self {
        let parts: Vec<&str> = role_str.split(':').collect();
        if parts.len() == 2 {
            Permission::new(parts[0], parts[1])
        } else {
            Permission::new(&role_str, "*")
        }
    }
}

/// JWT claims representation that's provider-agnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Issuer 	 the `iss` claim. See https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.1
    pub issuer: String,

    /// Subject - the `sub` claim. See https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.2
    pub subject: Uuid,

    /// Audiences - the `aud` claim. See https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.3
    pub audiences: Vec<String>,

    /// Expiration time - the `exp` claim. See https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.4
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub expires_at: Option<OffsetDateTime>,

    /// Not before time - the `nbf` claim. See https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.5
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub not_before: Option<OffsetDateTime>,

    /// Issued At - the `iat` claim. See https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.6
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub issued_at: Option<OffsetDateTime>,

    /// JWT ID - the `jti` claim. See https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.7
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt_id: Option<String>,

    /* modkit - specific claims */
    /// Tenant ID - the `tenant_id` claim
    pub tenant_id: Uuid,

    /// User roles
    #[serde(default)]
    pub permissions: Vec<Permission>,

    /// Additional provider-specific claims
    #[serde(flatten)]
    pub extras: serde_json::Map<String, serde_json::Value>,
}

impl Claims {
    /// Check if the token has expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at {
            OffsetDateTime::now_utc() >= exp
        } else {
            false
        }
    }

    /// Check if the token is valid yet (nbf check)
    #[must_use]
    pub fn is_valid_yet(&self) -> bool {
        if let Some(nbf) = self.not_before {
            OffsetDateTime::now_utc() >= nbf
        } else {
            true
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_expiration_check() {
        let mut claims = Claims {
            issuer: "test".to_owned(),
            subject: Uuid::new_v4(),
            audiences: vec!["api".to_owned()],
            expires_at: Some(OffsetDateTime::now_utc() + time::Duration::hours(1)),
            not_before: None,
            issued_at: None,
            jwt_id: None,
            tenant_id: Uuid::new_v4(),
            permissions: vec![],
            extras: serde_json::Map::new(),
        };

        assert!(!claims.is_expired());

        claims.expires_at = Some(OffsetDateTime::now_utc() - time::Duration::hours(1));
        assert!(claims.is_expired());
    }

    #[test]
    fn test_nbf_check() {
        let mut claims = Claims {
            subject: Uuid::new_v4(),
            issuer: "test".to_owned(),
            audiences: vec!["api".to_owned()],
            expires_at: None,
            not_before: Some(OffsetDateTime::now_utc() - time::Duration::hours(1)),
            issued_at: None,
            jwt_id: None,
            tenant_id: Uuid::new_v4(),
            permissions: vec![],
            extras: serde_json::Map::new(),
        };

        assert!(claims.is_valid_yet());

        claims.not_before = Some(OffsetDateTime::now_utc() + time::Duration::hours(1));
        assert!(!claims.is_valid_yet());
    }
}
