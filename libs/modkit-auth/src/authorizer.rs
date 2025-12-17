use crate::claims::Permission;
use crate::{claims::Claims, errors::AuthError, traits::PrimaryAuthorizer, types::SecRequirement};
use async_trait::async_trait;

/// Role-based authorizer that checks resource:action patterns
#[derive(Debug, Clone, Default)]
pub struct RoleAuthorizer;

impl RoleAuthorizer {
    /// Check if any role matches the requirement pattern
    fn check_role(claims: &Claims, requirement: &SecRequirement) -> bool {
        let required_permission = Permission::new(&requirement.resource, &requirement.action);

        // Check for exact match or wildcard patterns
        claims
            .permissions
            .iter()
            .any(|permission: &Permission| return permission.matches(&required_permission))
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn mock_claims(roles: Vec<String>) -> Claims {
        let permissions: Vec<Permission> = roles
            .into_iter()
            .map(|r| {
                if let Some((res, act)) = r.split_once(':') {
                    Permission::new(res, act)
                } else {
                    Permission::new(&r, "*")
                }
            })
            .collect();

        Claims {
            issuer: "test".to_owned(),
            subject: Uuid::new_v4(),
            audiences: vec!["api".to_owned()],
            expires_at: None,
            not_before: None,
            issued_at: None,
            jwt_id: None,
            tenant_id: Uuid::new_v4(),
            permissions,
            extras: serde_json::Map::new(),
        }
    }

    #[tokio::test]
    async fn test_exact_role_match() {
        let auth = RoleAuthorizer;
        let claims = mock_claims(vec!["users:read".to_owned()]);
        let req = SecRequirement::new("users", "read");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_resource_wildcard() {
        let auth = RoleAuthorizer;
        let claims = mock_claims(vec!["users:*".to_owned()]);
        let req = SecRequirement::new("users", "write");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_action_wildcard() {
        let auth = RoleAuthorizer;
        let claims = mock_claims(vec!["*:read".to_owned()]);
        let req = SecRequirement::new("posts", "read");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_full_wildcard() {
        let auth = RoleAuthorizer;
        let claims = mock_claims(vec!["*:*".to_owned()]);
        let req = SecRequirement::new("anything", "everything");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_no_matching_role() {
        let auth = RoleAuthorizer;
        let claims = mock_claims(vec!["posts:read".to_owned()]);
        let req = SecRequirement::new("users", "read");

        assert!(matches!(
            auth.check(&claims, &req).await,
            Err(AuthError::Forbidden)
        ));
    }
}
