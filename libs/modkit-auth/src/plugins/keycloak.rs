use crate::{
    claims::{Claims, Permission},
    claims_error::ClaimsError,
    plugin_traits::ClaimsPlugin,
    validation::{extract_audiences, extract_string, parse_timestamp, parse_uuid_from_value},
};
use serde_json::Value;

/// Keycloak-specific claims plugin
///
/// Handles Keycloak's specific claim structure:
/// - Roles from `realm_access.roles` and `resource_access.<client>.roles`
/// - Optional role prefix
/// - Tenant claim from configurable field (default: `tenant_id`)
/// - Handles Keycloak's audience validation via `aud`, `azp`, or `resource_access`
#[derive(Debug, Clone)]
pub struct KeycloakClaimsPlugin {
    /// Name of the tenant claim field (default: `tenant_id`)
    pub tenant_claim: String,

    /// Optional: client ID to extract roles from `resource_access`
    pub client_roles: Option<String>,

    /// Optional: prefix to add to all roles
    pub role_prefix: Option<String>,
}

impl Default for KeycloakClaimsPlugin {
    fn default() -> Self {
        Self {
            tenant_claim: "tenant_id".to_owned(),
            client_roles: None,
            role_prefix: None,
        }
    }
}

impl KeycloakClaimsPlugin {
    /// Create a new Keycloak plugin with custom configuration
    pub fn new(
        tenant_claim: impl Into<String>,
        client_roles: Option<String>,
        role_prefix: Option<String>,
    ) -> Self {
        Self {
            tenant_claim: tenant_claim.into(),
            client_roles,
            role_prefix,
        }
    }

    /// Extract permissions from Keycloak's complex role structure
    fn extract_permissions(&self, raw: &Value) -> Vec<Permission> {
        let mut roles = Vec::new();

        // 1. Check for top-level "roles" array (simplified format)
        if let Some(Value::Array(arr)) = raw.get("roles") {
            roles.extend(
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(ToString::to_string),
            );
        }

        // 2. Extract from realm_access.roles
        if let Some(Value::Object(realm)) = raw.get("realm_access") {
            if let Some(Value::Array(arr)) = realm.get("roles") {
                roles.extend(
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(ToString::to_string),
                );
            }
        }

        // 3. Extract from resource_access.<client>.roles
        if let Some(client_id) = &self.client_roles {
            if let Some(Value::Object(resource_access)) = raw.get("resource_access") {
                if let Some(Value::Object(client)) = resource_access.get(client_id) {
                    if let Some(Value::Array(arr)) = client.get("roles") {
                        roles.extend(
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(ToString::to_string),
                        );
                    }
                }
            }
        }

        // Apply role prefix if configured
        if let Some(prefix) = &self.role_prefix {
            roles = roles.into_iter().map(|r| format!("{prefix}:{r}")).collect();
        }

        // Deduplicate
        roles.sort();
        roles.dedup();

        // Convert roles to permissions (resource_pattern:action format)
        roles
            .into_iter()
            .filter_map(|role| {
                // Try to parse as "resource:action" format
                if let Some(pos) = role.rfind(':') {
                    Permission::builder()
                        .resource_pattern(&role[..pos])
                        .action(&role[pos + 1..])
                        .build()
                        .ok()
                } else {
                    // Treat as resource with wildcard action
                    Permission::builder()
                        .resource_pattern(&role)
                        .action("*")
                        .build()
                        .ok()
                }
            })
            .collect()
    }
}

impl ClaimsPlugin for KeycloakClaimsPlugin {
    fn name(&self) -> &'static str {
        "keycloak"
    }

    fn normalize(&self, raw: &Value) -> Result<Claims, ClaimsError> {
        // 1. Extract subject (required, must be UUID)
        let subject = raw
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

        // 6. Extract issued-at time
        let issued_at = raw
            .get("iat")
            .map(|v| parse_timestamp(v, "iat"))
            .transpose()?;

        // 7. Extract JWT ID
        let jwt_id = raw
            .get("jti")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        // 8. Extract tenant_id (required, must be UUID)
        let tenant_id = raw
            .get(&self.tenant_claim)
            .ok_or_else(|| ClaimsError::MissingClaim(self.tenant_claim.clone()))
            .and_then(|v| parse_uuid_from_value(v, &self.tenant_claim))?;

        // 9. Extract permissions using Keycloak-specific logic
        let permissions = self.extract_permissions(raw);

        // 10. Collect extra claims (excluding standard ones)
        let mut extras = serde_json::Map::new();
        let standard_fields = [
            "sub",
            "iss",
            "aud",
            "exp",
            "nbf",
            "iat",
            "jti",
            "azp",
            &self.tenant_claim,
            "roles",
            "realm_access",
            "resource_access",
        ];

        if let Value::Object(obj) = raw {
            for (key, value) in obj {
                if !standard_fields.contains(&key.as_str()) {
                    extras.insert(key.clone(), value.clone());
                }
            }
        }

        // Add email if present
        if let Some(email) = raw.get("email") {
            extras.insert("email".to_owned(), email.clone());
        }

        // Add preferred_username if present
        if let Some(username) = raw.get("preferred_username") {
            extras.insert("preferred_username".to_owned(), username.clone());
        }

        // Add name if present
        if let Some(name) = raw.get("name") {
            extras.insert("name".to_owned(), name.clone());
        }

        Ok(Claims {
            issuer,
            subject,
            audiences,
            expires_at,
            not_before,
            issued_at,
            jwt_id,
            tenant_id,
            permissions,
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
    fn test_keycloak_plugin_normalize() {
        let plugin = KeycloakClaimsPlugin::default();

        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let claims = json!({
            "iss": "https://kc.example.com/realms/test",
            "sub": user_id.to_string(),
            "aud": "modkit-api",
            "exp": 9999999999i64,
            "tenant_id": tenant_id.to_string(),
            "realm_access": {
                "roles": ["users:read", "admin:write"]
            },
            "email": "test@example.com"
        });

        let normalized = plugin.normalize(&claims).unwrap();

        assert_eq!(normalized.subject, user_id);
        assert_eq!(normalized.issuer, "https://kc.example.com/realms/test");
        assert_eq!(normalized.audiences, vec!["modkit-api"]);
        assert_eq!(normalized.tenant_id, tenant_id);
        assert_eq!(normalized.permissions.len(), 2);
        assert_eq!(
            normalized.extras.get("email").unwrap().as_str().unwrap(),
            "test@example.com"
        );
    }

    #[test]
    fn test_keycloak_extract_permissions_with_client() {
        let plugin = KeycloakClaimsPlugin::new("tenant_id", Some("modkit-api".to_owned()), None);

        let claims = json!({
            "realm_access": {
                "roles": ["realm:role"]
            },
            "resource_access": {
                "modkit-api": {
                    "roles": ["api:role"]
                }
            }
        });

        let permissions = plugin.extract_permissions(&claims);
        assert_eq!(permissions.len(), 2);
    }

    #[test]
    fn test_keycloak_extract_permissions_with_prefix() {
        let plugin = KeycloakClaimsPlugin::new("tenant_id", None, Some("kc".to_owned()));

        let claims = json!({
            "realm_access": {
                "roles": ["admin", "user"]
            }
        });

        let permissions = plugin.extract_permissions(&claims);
        assert_eq!(permissions.len(), 2);
        // Prefixed roles become "kc:admin" and "kc:user", parsed as resource:action
        assert!(permissions
            .iter()
            .any(|p| p.resource_pattern() == "kc" && p.action() == "admin"));
        assert!(permissions
            .iter()
            .any(|p| p.resource_pattern() == "kc" && p.action() == "user"));
    }

    #[test]
    fn test_keycloak_missing_subject_fails() {
        let plugin = KeycloakClaimsPlugin::default();

        let claims = json!({
            "iss": "https://kc.example.com/realms/test",
            "aud": "modkit-api"
        });

        let result = plugin.normalize(&claims);
        assert!(matches!(result, Err(ClaimsError::MissingClaim(_))));
    }

    #[test]
    fn test_keycloak_invalid_uuid_fails() {
        let plugin = KeycloakClaimsPlugin::default();

        let claims = json!({
            "iss": "https://kc.example.com/realms/test",
            "sub": "not-a-uuid",
            "aud": "modkit-api"
        });

        let result = plugin.normalize(&claims);
        assert!(matches!(
            result,
            Err(ClaimsError::InvalidClaimFormat { .. })
        ));
    }
}
