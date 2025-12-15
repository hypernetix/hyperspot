use crate::{
    claims::Claims,
    claims_error::ClaimsError,
    plugin_traits::ClaimsPlugin,
    validation::{
        extract_audiences, extract_string, parse_timestamp, parse_uuid_array_from_value,
        parse_uuid_from_value,
    },
};
use serde_json::Value;

/// Generic OIDC claims plugin
///
/// Handles standard OIDC claims with configurable field names.
/// This plugin serves as a fallback for any OIDC-compliant provider
/// that doesn't need special handling.
#[derive(Debug, Clone)]
pub struct GenericOidcPlugin {
    /// Name of the tenant claim field (default: "tenants")
    pub tenant_claim: String,

    /// Name of the roles claim field (default: "roles")
    pub roles_claim: String,
}

impl Default for GenericOidcPlugin {
    fn default() -> Self {
        Self {
            tenant_claim: "tenants".to_owned(),
            roles_claim: "roles".to_owned(),
        }
    }
}

impl GenericOidcPlugin {
    /// Create a new generic OIDC plugin with custom configuration
    pub fn new(tenant_claim: impl Into<String>, roles_claim: impl Into<String>) -> Self {
        Self {
            tenant_claim: tenant_claim.into(),
            roles_claim: roles_claim.into(),
        }
    }

    /// Extract roles from the configured roles claim
    fn extract_roles(&self, raw: &Value) -> Vec<String> {
        raw.get(&self.roles_claim)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(ToString::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl ClaimsPlugin for GenericOidcPlugin {
    fn name(&self) -> &'static str {
        "generic-oidc"
    }

    fn normalize(&self, raw: &Value) -> Result<Claims, ClaimsError> {
        // 1. Extract subject (required, must be UUID)
        let sub = raw
            .get("sub")
            .ok_or_else(|| ClaimsError::MissingClaim("sub".to_owned()))
            .and_then(|v| parse_uuid_from_value(v, "sub"))?;

        // 2. Extract issuer (required)
        let issuer = raw
            .get("iss")
            .ok_or_else(|| ClaimsError::MissingClaim("iss".to_owned()))
            .and_then(|v| extract_string(v, "iss"))?;

        // 3. Extract audiences (handle string or array)
        let audiences = raw.get("aud").map(extract_audiences).unwrap_or_default();

        // 4. Extract expiration time
        let expires_at = raw
            .get("exp")
            .map(|v| parse_timestamp(v, "exp"))
            .transpose()?;

        // 5. Extract not-before time
        let not_before = raw
            .get("nbf")
            .map(|v| parse_timestamp(v, "nbf"))
            .transpose()?;

        // 6. Extract tenants (optional, must be UUIDs)
        let tenants = raw
            .get(&self.tenant_claim)
            .map(|v| parse_uuid_array_from_value(v, &self.tenant_claim))
            .transpose()?
            .unwrap_or_default();

        // 7. Extract roles from configured field
        let roles = self.extract_roles(raw);

        // 8. Collect extra claims (excluding standard ones)
        let mut extras = serde_json::Map::new();
        let standard_fields = [
            "sub",
            "iss",
            "aud",
            "exp",
            "nbf",
            "iat",
            "jti",
            &self.tenant_claim,
            &self.roles_claim,
        ];

        if let Value::Object(obj) = raw {
            for (key, value) in obj {
                if !standard_fields.contains(&key.as_str()) {
                    extras.insert(key.clone(), value.clone());
                }
            }
        }

        // Explicitly add common OIDC profile claims to extras
        for field in [
            "email",
            "name",
            "preferred_username",
            "given_name",
            "family_name",
            "picture",
        ] {
            if let Some(value) = raw.get(field) {
                extras.insert(field.to_owned(), value.clone());
            }
        }

        Ok(Claims {
            sub,
            issuer,
            audiences,
            expires_at,
            not_before,
            tenants,
            roles,
            extras,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unreadable_literal)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn test_generic_oidc_normalize() {
        let plugin = GenericOidcPlugin::default();

        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let claims = json!({
            "iss": "https://auth.example.com",
            "sub": user_id.to_string(),
            "aud": ["api", "ui"],
            "exp": 9999999999i64,
            "roles": ["user", "admin"],
            "tenants": [tenant_id.to_string()],
            "email": "test@example.com",
            "name": "Test User"
        });

        let normalized = plugin.normalize(&claims).unwrap();

        assert_eq!(normalized.sub, user_id);
        assert_eq!(normalized.issuer, "https://auth.example.com");
        assert_eq!(normalized.audiences, vec!["api", "ui"]);
        assert_eq!(normalized.tenants, vec![tenant_id]);
        assert_eq!(normalized.roles, vec!["user", "admin"]);
        assert_eq!(
            normalized.extras.get("email").unwrap().as_str().unwrap(),
            "test@example.com"
        );
        assert_eq!(
            normalized.extras.get("name").unwrap().as_str().unwrap(),
            "Test User"
        );
    }

    #[test]
    fn test_generic_oidc_custom_claims() {
        let plugin = GenericOidcPlugin::new("organizations", "permissions");

        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let claims = json!({
            "iss": "https://auth.example.com",
            "sub": user_id.to_string(),
            "aud": "api",
            "permissions": ["read", "write"],
            "organizations": [org_id.to_string()]
        });

        let normalized = plugin.normalize(&claims).unwrap();

        assert_eq!(normalized.tenants, vec![org_id]);
        assert_eq!(normalized.roles, vec!["read", "write"]);
    }

    #[test]
    fn test_generic_oidc_missing_subject_fails() {
        let plugin = GenericOidcPlugin::default();

        let claims = json!({
            "iss": "https://auth.example.com",
            "aud": "api"
        });

        let result = plugin.normalize(&claims);
        assert!(matches!(result, Err(ClaimsError::MissingClaim(_))));
    }

    #[test]
    fn test_generic_oidc_handles_string_audience() {
        let plugin = GenericOidcPlugin::default();

        let user_id = Uuid::new_v4();

        let claims = json!({
            "iss": "https://auth.example.com",
            "sub": user_id.to_string(),
            "aud": "api",  // String instead of array
            "exp": 9999999999i64
        });

        let normalized = plugin.normalize(&claims).unwrap();
        assert_eq!(normalized.audiences, vec!["api"]);
    }
}
