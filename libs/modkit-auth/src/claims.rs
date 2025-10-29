use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// JWT claims representation that's provider-agnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID) - must be a UUID
    pub sub: Uuid,

    /// Issuer
    pub issuer: String,

    /// Audiences (can be multiple)
    pub audiences: Vec<String>,

    /// Expiration time
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub expires_at: Option<OffsetDateTime>,

    /// Not before time
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub not_before: Option<OffsetDateTime>,

    /// Tenant IDs (all must be UUIDs)
    #[serde(default)]
    pub tenants: Vec<Uuid>,

    /// User roles
    #[serde(default)]
    pub roles: Vec<String>,

    /// Additional provider-specific claims
    #[serde(flatten)]
    pub extras: serde_json::Map<String, serde_json::Value>,
}

impl Claims {
    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at {
            OffsetDateTime::now_utc() >= exp
        } else {
            false
        }
    }

    /// Check if the token is valid yet (nbf check)
    pub fn is_valid_yet(&self) -> bool {
        if let Some(nbf) = self.not_before {
            OffsetDateTime::now_utc() >= nbf
        } else {
            true
        }
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if user has access to a specific tenant
    pub fn has_tenant(&self, tenant_id: &Uuid) -> bool {
        self.tenants.contains(tenant_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expiration_check() {
        let mut claims = Claims {
            sub: Uuid::new_v4(),
            issuer: "test".to_string(),
            audiences: vec!["api".to_string()],
            expires_at: Some(OffsetDateTime::now_utc() + time::Duration::hours(1)),
            not_before: None,
            tenants: vec![],
            roles: vec![],
            extras: serde_json::Map::new(),
        };

        assert!(!claims.is_expired());

        claims.expires_at = Some(OffsetDateTime::now_utc() - time::Duration::hours(1));
        assert!(claims.is_expired());
    }

    #[test]
    fn test_nbf_check() {
        let mut claims = Claims {
            sub: Uuid::new_v4(),
            issuer: "test".to_string(),
            audiences: vec!["api".to_string()],
            expires_at: None,
            not_before: Some(OffsetDateTime::now_utc() - time::Duration::hours(1)),
            tenants: vec![],
            roles: vec![],
            extras: serde_json::Map::new(),
        };

        assert!(claims.is_valid_yet());

        claims.not_before = Some(OffsetDateTime::now_utc() + time::Duration::hours(1));
        assert!(!claims.is_valid_yet());
    }

    #[test]
    fn test_role_check() {
        let claims = Claims {
            sub: Uuid::new_v4(),
            issuer: "test".to_string(),
            audiences: vec!["api".to_string()],
            expires_at: None,
            not_before: None,
            tenants: vec![],
            roles: vec!["admin".to_string(), "user".to_string()],
            extras: serde_json::Map::new(),
        };

        assert!(claims.has_role("admin"));
        assert!(claims.has_role("user"));
        assert!(!claims.has_role("superuser"));
    }
}
