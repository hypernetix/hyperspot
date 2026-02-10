use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Represents a permission from JWT claims.
/// Serializes to format: `"{tenant_id}:{resource_pattern}:{resource_id}:{action}"`
/// where `tenant_id` and `resource_id` are "*" if None.
/// This is compatible with `modkit_security::Permission`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Permission {
    /// Optional tenant ID the permission applies to
    /// e.g., a specific tenant UUID
    tenant_id: Option<Uuid>,

    /// A pattern that can include wildcards to match multiple resources
    /// examples:
    ///   - `gts.x.core.events.topic.v1~vendor.*`
    ///   - `gts.x.mod.v1~x.file-parser.v1`
    resource_pattern: String,

    /// Optional specific resource ID the permission applies to
    /// e.g., a specific topic or file UUID
    resource_id: Option<Uuid>,

    /// The action that can be performed on the resource
    /// e.g., "publish", "subscribe", "edit"
    action: String,
}

impl Permission {
    #[must_use]
    pub fn builder() -> PermissionBuilder {
        PermissionBuilder::default()
    }

    #[must_use]
    pub fn tenant_id(&self) -> Option<Uuid> {
        self.tenant_id
    }

    #[must_use]
    pub fn resource_pattern(&self) -> &str {
        &self.resource_pattern
    }

    #[must_use]
    pub fn resource_id(&self) -> Option<Uuid> {
        self.resource_id
    }

    #[must_use]
    pub fn action(&self) -> &str {
        &self.action
    }
}

impl serde::Serialize for Permission {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let tenant_id_str = self
            .tenant_id
            .map_or_else(|| "*".to_owned(), |id| id.to_string());
        let resource_id_str = self
            .resource_id
            .map_or_else(|| "*".to_owned(), |id| id.to_string());
        let s = format!(
            "{}:{}:{}:{}",
            tenant_id_str, self.resource_pattern, resource_id_str, self.action
        );
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for Permission {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.splitn(4, ':').collect();

        if parts.len() != 4 {
            return Err(serde::de::Error::custom(format!(
                "Expected format 'tenant_id:resource_pattern:resource_id:action', got: {s}"
            )));
        }

        let tenant_id = if parts[0] == "*" {
            None
        } else {
            Some(Uuid::parse_str(parts[0]).map_err(serde::de::Error::custom)?)
        };

        let resource_id = if parts[2] == "*" {
            None
        } else {
            Some(Uuid::parse_str(parts[2]).map_err(serde::de::Error::custom)?)
        };

        let action = parts[3];
        if !action
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(serde::de::Error::custom(format!(
                "Action must contain only alphanumeric characters and underscores, got: {action}"
            )));
        }

        Ok(Permission {
            tenant_id,
            resource_pattern: parts[1].to_owned(),
            resource_id,
            action: action.to_owned(),
        })
    }
}

/// Builder for creating `Permission` instances
#[derive(Default)]
pub struct PermissionBuilder {
    tenant_id: Option<Uuid>,
    resource_pattern: Option<String>,
    resource_id: Option<Uuid>,
    action: Option<String>,
}

impl PermissionBuilder {
    #[must_use]
    pub fn tenant_id(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    #[must_use]
    pub fn resource_pattern(mut self, resource_pattern: &str) -> Self {
        self.resource_pattern = Some(resource_pattern.to_owned());
        self
    }

    #[must_use]
    pub fn resource_id(mut self, resource_id: Uuid) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    #[must_use]
    pub fn action(mut self, action: &str) -> Self {
        self.action = Some(action.to_owned());
        self
    }

    /// Build the permission
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `resource_pattern` is not set
    /// - `action` is not set
    /// - `action` contains characters other than alphanumeric, underscore, or wildcard (*)
    pub fn build(self) -> Result<Permission, PermissionBuildError> {
        let resource_pattern = self
            .resource_pattern
            .ok_or(PermissionBuildError::MissingResourcePattern)?;

        let action = self.action.ok_or(PermissionBuildError::MissingAction)?;

        // Validate action contains only alphanumeric characters, underscores, or wildcard
        if !action
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '*')
        {
            return Err(PermissionBuildError::InvalidAction(action));
        }

        Ok(Permission {
            tenant_id: self.tenant_id,
            resource_pattern,
            resource_id: self.resource_id,
            action,
        })
    }
}

/// Error type for `PermissionBuilder`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionBuildError {
    MissingResourcePattern,
    MissingAction,
    InvalidAction(String),
}

impl std::fmt::Display for PermissionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingResourcePattern => write!(f, "resource_pattern is required"),
            Self::MissingAction => write!(f, "action is required"),
            Self::InvalidAction(action) => write!(
                f,
                "Action must contain only alphanumeric characters and underscores, got: {action}"
            ),
        }
    }
}

impl std::error::Error for PermissionBuildError {}

/// JWT claims representation that's provider-agnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Issuer - the `iss` claim. See <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.1>
    pub issuer: String,

    /// Subject - the `sub` claim. See <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.2>
    pub subject: Uuid,

    /// Audiences - the `aud` claim. See <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.3>
    pub audiences: Vec<String>,

    /// Expiration time - the `exp` claim. See <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.4>
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub expires_at: Option<OffsetDateTime>,

    /// Not before time - the `nbf` claim. See <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.5>
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub not_before: Option<OffsetDateTime>,

    /// Issued At - the `iat` claim. See <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.6>
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub issued_at: Option<OffsetDateTime>,

    /// JWT ID - the `jti` claim. See <https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.7>
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
