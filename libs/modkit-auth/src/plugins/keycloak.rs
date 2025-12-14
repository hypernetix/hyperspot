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

/// Keycloak-specific claims plugin
///
/// Handles Keycloak's specific claim structure:
/// - Roles from `realm_access.roles` and `resource_access.<client>.roles`
/// - Optional role prefix
/// - Tenant claim from configurable field (default: "tenants")
/// - Handles Keycloak's audience validation via `aud`, `azp`, or `resource_access`
#[derive(Debug, Clone)]
pub struct KeycloakClaimsPlugin {
    /// Name of the tenant claim field (default: "tenants")
    pub tenant_claim: String,

    /// Optional: client ID to extract roles from resource_access
    pub client_roles: Option<String>,

    /// Optional: prefix to add to all roles
    pub role_prefix: Option<String>,
}

impl Default for KeycloakClaimsPlugin {
    fn default() -> Self {
        Self {
            tenant_claim: "tenants".to_string(),
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

    /// Extract roles from Keycloak's complex role structure
    fn extract_roles(&self, raw: &Value) -> Vec<String> {
        let mut roles = Vec::new();

        // 1. Check for top-level "roles" array (simplified format)
        if let Some(Value::Array(arr)) = raw.get("roles") {
            roles.extend(arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()));
        }

        // 2. Extract from realm_access.roles
        if let Some(Value::Object(realm)) = raw.get("realm_access") {
            if let Some(Value::Array(arr)) = realm.get("roles") {
                roles.extend(arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()));
            }
        }

        // 3. Extract from resource_access.<client>.roles
        if let Some(client_id) = &self.client_roles {
            if let Some(Value::Object(resource_access)) = raw.get("resource_access") {
                if let Some(Value::Object(client)) = resource_access.get(client_id) {
                    if let Some(Value::Array(arr)) = client.get("roles") {
                        roles.extend(arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()));
                    }
                }
            }
        }

        // Apply role prefix if configured
        if let Some(prefix) = &self.role_prefix {
            roles = roles
                .into_iter()
                .map(|r| format!("{}:{}", prefix, r))
                .collect();
        }

        // Deduplicate
        roles.sort();
        roles.dedup();

        roles
    }
}

impl ClaimsPlugin for KeycloakClaimsPlugin {
    fn name(&self) -> &'static str {
        "keycloak"
    }

    fn normalize(&self, raw: &Value) -> Result<Claims, ClaimsError> {
        // 1. Extract subject (required, must be UUID)
        let sub = raw
            .get("sub")
            .ok_or_else(|| ClaimsError::MissingClaim("sub".to_string()))
            .and_then(|v| parse_uuid_from_value(v, "sub"))?;

        // 2. Extract issuer (required)
        let issuer = raw
            .get("iss")
            .ok_or_else(|| ClaimsError::MissingClaim("iss".to_string()))
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

        // 7. Extract roles using Keycloak-specific logic
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
            extras.insert("email".to_string(), email.clone());
        }

        // Add preferred_username if present
        if let Some(username) = raw.get("preferred_username") {
            extras.insert("preferred_username".to_string(), username.clone());
        }

        // Add name if present
        if let Some(name) = raw.get("name") {
            extras.insert("name".to_string(), name.clone());
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
            "tenants": [tenant_id.to_string()],
            "realm_access": {
                "roles": ["user", "admin"]
            },
            "email": "test@example.com"
        });

        let normalized = plugin.normalize(&claims).unwrap();

        assert_eq!(normalized.sub, user_id);
        assert_eq!(normalized.issuer, "https://kc.example.com/realms/test");
        assert_eq!(normalized.audiences, vec!["modkit-api"]);
        assert_eq!(normalized.tenants, vec![tenant_id]);
        assert_eq!(normalized.roles, vec!["admin", "user"]);
        assert_eq!(
            normalized.extras.get("email").unwrap().as_str().unwrap(),
            "test@example.com"
        );
    }

    #[test]
    fn test_keycloak_extract_roles_with_client() {
        let plugin = KeycloakClaimsPlugin::new("tenants", Some("modkit-api".to_string()), None);

        let claims = json!({
            "realm_access": {
                "roles": ["realm-role"]
            },
            "resource_access": {
                "modkit-api": {
                    "roles": ["api-role"]
                }
            }
        });

        let roles = plugin.extract_roles(&claims);
        assert!(roles.contains(&"realm-role".to_string()));
        assert!(roles.contains(&"api-role".to_string()));
    }

    #[test]
    fn test_keycloak_extract_roles_with_prefix() {
        let plugin = KeycloakClaimsPlugin::new("tenants", None, Some("kc".to_string()));

        let claims = json!({
            "realm_access": {
                "roles": ["admin", "user"]
            }
        });

        let roles = plugin.extract_roles(&claims);
        assert_eq!(roles, vec!["kc:admin", "kc:user"]);
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
