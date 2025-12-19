use crate::{claims::Claims, errors::AuthError, traits::PrimaryAuthorizer, types::SecRequirement};
use async_trait::async_trait;

/// Role-based authorizer that checks resource:action patterns
#[derive(Debug, Clone, Default)]
pub struct RoleAuthorizer;

impl RoleAuthorizer {
    /// Check if any permission matches the requirement pattern
    fn check_permission(claims: &Claims, requirement: &SecRequirement) -> bool {
        claims.permissions.iter().any(|perm| {
            let resource = perm.resource_pattern();
            let action = perm.action();

            // Exact match
            if resource == requirement.resource && action == requirement.action {
                return true;
            }

            // Resource wildcard: "users:*"
            if resource == requirement.resource && action == "*" {
                return true;
            }

            // Action wildcard: "*:read"
            if resource == "*" && action == requirement.action {
                return true;
            }

            // Full wildcard: "*:*"
            if resource == "*" && action == "*" {
                return true;
            }

            false
        })
    }
}

#[async_trait]
impl PrimaryAuthorizer for RoleAuthorizer {
    async fn check(&self, claims: &Claims, requirement: &SecRequirement) -> Result<(), AuthError> {
        if Self::check_permission(claims, requirement) {
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
    use crate::claims::Permission;
    use uuid::Uuid;

    fn mock_claims(permissions: Vec<Permission>) -> Claims {
        Claims {
            issuer: "test".to_owned(),
            subject: Uuid::new_v4(),
            audiences: vec![],
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
        let perm = Permission::builder()
            .resource_pattern("users")
            .action("read")
            .build()
            .unwrap();
        let claims = mock_claims(vec![perm]);
        let req = SecRequirement::new("users", "read");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_resource_wildcard() {
        let auth = RoleAuthorizer;
        let perm = Permission::builder()
            .resource_pattern("users")
            .action("*")
            .build()
            .unwrap();
        let claims = mock_claims(vec![perm]);
        let req = SecRequirement::new("users", "write");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_action_wildcard() {
        let auth = RoleAuthorizer;
        let perm = Permission::builder()
            .resource_pattern("*")
            .action("read")
            .build()
            .unwrap();
        let claims = mock_claims(vec![perm]);
        let req = SecRequirement::new("posts", "read");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_full_wildcard() {
        let auth = RoleAuthorizer;
        let perm = Permission::builder()
            .resource_pattern("*")
            .action("*")
            .build()
            .unwrap();
        let claims = mock_claims(vec![perm]);
        let req = SecRequirement::new("anything", "everything");

        assert!(auth.check(&claims, &req).await.is_ok());
    }

    #[tokio::test]
    async fn test_no_matching_role() {
        let auth = RoleAuthorizer;
        let perm = Permission::builder()
            .resource_pattern("posts")
            .action("read")
            .build()
            .unwrap();
        let claims = mock_claims(vec![perm]);
        let req = SecRequirement::new("users", "read");

        assert!(matches!(
            auth.check(&claims, &req).await,
            Err(AuthError::Forbidden)
        ));
    }
}
