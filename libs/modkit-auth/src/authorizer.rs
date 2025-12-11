use crate::{claims::Claims, errors::AuthError, traits::PrimaryAuthorizer, types::SecRequirement};
use async_trait::async_trait;

/// Role-based authorizer that checks resource:action patterns
#[derive(Debug, Clone, Default)]
pub struct RoleAuthorizer;

impl RoleAuthorizer {
    /// Check if any role matches the requirement pattern
    fn check_role(claims: &Claims, requirement: &SecRequirement) -> bool {
        let required_role = format!("{}:{}", requirement.resource, requirement.action);

        // Check for exact match or wildcard patterns
        claims.roles.iter().any(|role| {
            // Exact match: "users:read"
            if role == &required_role {
                return true;
            }

            // Resource wildcard: "users:*"
            if role == &format!("{}:*", requirement.resource) {
                return true;
            }

            // Action wildcard: "*:read"
            if role == &format!("*:{}", requirement.action) {
                return true;
            }

            // Full wildcard: "*:*"
            if role == "*:*" {
                return true;
            }

            false
        })
    }
}

#[async_trait]
impl PrimaryAuthorizer for RoleAuthorizer {
    async fn check(&self, claims: &Claims, requirement: &SecRequirement) -> Result<(), AuthError> {
        if Self::check_role(claims, requirement) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn mock_claims(roles: Vec<String>) -> Claims {
        Claims {
            sub: Uuid::new_v4(),
            issuer: "test".to_string(),
            audiences: vec![],
            expires_at: None,
            not_before: None,
            tenants: vec![],
            roles,
            extras: serde_json::Map::new(),
        }
    }

    #[tokio::test]
    async fn test_exact_role_match() {
        let auth = RoleAuthorizer;
        let claims = mock_claims(vec!["users:read".to_string()]);
        let req = SecRequirement::new("users", "read");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_resource_wildcard() {
        let auth = RoleAuthorizer;
        let claims = mock_claims(vec!["users:*".to_string()]);
        let req = SecRequirement::new("users", "write");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_action_wildcard() {
        let auth = RoleAuthorizer;
        let claims = mock_claims(vec!["*:read".to_string()]);
        let req = SecRequirement::new("posts", "read");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_full_wildcard() {
        let auth = RoleAuthorizer;
        let claims = mock_claims(vec!["*:*".to_string()]);
        let req = SecRequirement::new("anything", "everything");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_no_matching_role() {
        let auth = RoleAuthorizer;
        let claims = mock_claims(vec!["posts:read".to_string()]);
        let req = SecRequirement::new("users", "read");

        assert!(matches!(
            auth.check(&claims, &req).await,
            Err(AuthError::Forbidden)
        ));
    }
}
