#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test support utilities for `users_info` integration tests.
//!
//! Provides helper functions for creating security contexts, test databases,
//! and seeding test data.

#![allow(dead_code)] // Support module provides utilities that may not all be used

use hs_tenant_resolver_sdk::{
    AccessOptions, TenantFilter, TenantId, TenantInfo, TenantResolverError,
    TenantResolverGatewayClient, TenantStatus,
};
use modkit_db::secure::{AccessScope, SecureConn, SecurityCtx, Subject};
use modkit_security::SecurityContext;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use time::OffsetDateTime;
use uuid::Uuid;

use user_info_sdk::User;
use users_info::domain::{
    error::DomainError,
    events::UserDomainEvent,
    ports::{AuditPort, EventPublisher},
};

/// Create a security context that allows access to specific tenants.
///
/// Uses a random subject ID for testing purposes.
#[must_use]
pub fn ctx_allow_tenants(tenants: &[Uuid]) -> SecurityCtx {
    let subject = Subject::new(Uuid::new_v4());
    #[allow(deprecated)]
    SecurityCtx::new(AccessScope::tenants_only(tenants.to_vec()), subject)
}

/// Create a security context that allows access to specific resources.
///
/// Uses a random subject ID for testing purposes.
#[must_use]
pub fn ctx_allow_resources(resources: &[Uuid]) -> SecurityCtx {
    let subject = Subject::new(Uuid::new_v4());
    #[allow(deprecated)]
    SecurityCtx::new(AccessScope::resources_only(resources.to_vec()), subject)
}

/// Create a security context with a specific subject ID and tenant access.
///
/// Useful when you need to test `owner_id` or subject-specific behavior.
#[must_use]
pub fn ctx_with_subject(subject_id: Uuid, tenants: &[Uuid]) -> SecurityCtx {
    let subject = Subject::new(subject_id);
    #[allow(deprecated)]
    SecurityCtx::new(AccessScope::tenants_only(tenants.to_vec()), subject)
}

/// Create a deny-all security context.
///
/// This context will deny access to all data (empty scope).
#[must_use]
pub fn ctx_deny_all() -> SecurityCtx {
    let subject = Subject::new(Uuid::new_v4());
    #[allow(deprecated)]
    SecurityCtx::new(AccessScope::default(), subject)
}

/// Create a fresh in-memory `SQLite` database with migrations applied.
///
/// Each call creates a new isolated database for testing.
///
/// # Panics
/// Panics if the database connection or migrations fail.
pub async fn inmem_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory database");

    users_info::infra::storage::migrations::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    db
}

/// Create a `SecureConn` wrapped around an in-memory database.
///
/// This is the primary interface for secure database operations in tests.
pub async fn inmem_secure_db() -> SecureConn {
    SecureConn::new(inmem_db().await)
}

/// Seed a test user directly into the database (bypassing security for setup).
///
/// Returns the created user for use in tests.
///
/// # Note
/// This uses the raw `DatabaseConnection` to bypass security for test setup.
/// In production code, all inserts should go through `SecureConn`.
///
/// # Safety
/// This function requires access to the raw `DatabaseConnection` for test seeding.
/// Use `inmem_db()` to get the connection, then wrap it with `SecureConn::new()`
/// after seeding is complete.
///
/// # Panics
/// Panics if the database insert fails.
pub async fn seed_user(
    db: &DatabaseConnection,
    id: Uuid,
    tenant_id: Uuid,
    email: &str,
    display_name: &str,
) -> User {
    use sea_orm::{ActiveModelTrait, Set};
    use users_info::infra::storage::entity::ActiveModel;

    let now = OffsetDateTime::now_utc();
    let am = ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        email: Set(email.to_owned()),
        display_name: Set(display_name.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let model = am.insert(db).await.expect("Failed to seed user");

    model.into()
}

/// Create a test database, seed data, and return both the `SecureConn` and raw connection.
///
/// Use this when you need to seed data and then create a repository.
/// The raw connection can be used for seeding (bypassing security), then wrap
/// it with `SecureConn` for the repository.
pub async fn setup_test_db_with_users(
    users: Vec<(Uuid, Uuid, &str, &str)>,
) -> (DatabaseConnection, SecureConn) {
    let db = inmem_db().await;

    // Seed users using raw connection
    for (id, tenant_id, email, display_name) in users {
        seed_user(&db, id, tenant_id, email, display_name).await;
    }

    // Return both raw and secure connections
    let sec = SecureConn::new(db.clone());
    (db, sec)
}

/// Mock audit port for tests - always succeeds silently.
#[derive(Clone)]
pub struct MockAuditPort;

#[async_trait::async_trait]
impl AuditPort for MockAuditPort {
    async fn get_user_access(&self, _id: Uuid) -> Result<(), DomainError> {
        Ok(())
    }

    async fn notify_user_created(&self) -> Result<(), DomainError> {
        Ok(())
    }
}

/// Mock event publisher for tests - discards all events.
#[derive(Clone)]
pub struct MockEventPublisher;

impl EventPublisher<UserDomainEvent> for MockEventPublisher {
    fn publish(&self, _event: &UserDomainEvent) {
        // Discard events in tests
    }
}

/// Mock tenant resolver for tests - returns only the caller's tenant.
#[derive(Clone)]
pub struct MockTenantResolver;

impl MockTenantResolver {
    fn matches_filter(filter: Option<&TenantFilter>) -> bool {
        // Mock always returns Active status
        if let Some(f) = filter {
            if !f.status.is_empty() && !f.status.contains(&TenantStatus::Active) {
                return false;
            }
        }
        true
    }
}

#[async_trait::async_trait]
impl TenantResolverGatewayClient for MockTenantResolver {
    async fn get_tenant(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
    ) -> Result<TenantInfo, TenantResolverError> {
        // Return tenant info if requesting own tenant
        if id == ctx.tenant_id() {
            Ok(TenantInfo {
                id,
                name: format!("Tenant {id}"),
                status: TenantStatus::Active,
                tenant_type: None,
            })
        } else {
            Err(TenantResolverError::TenantNotFound { tenant_id: id })
        }
    }

    async fn can_access(
        &self,
        ctx: &SecurityContext,
        target: TenantId,
        _options: Option<&AccessOptions>,
    ) -> Result<bool, TenantResolverError> {
        // Return error if target doesn't exist (not matching context tenant)
        if target != ctx.tenant_id() {
            return Err(TenantResolverError::TenantNotFound { tenant_id: target });
        }
        // Allow self-access
        Ok(true)
    }

    async fn get_accessible_tenants(
        &self,
        ctx: &SecurityContext,
        filter: Option<&TenantFilter>,
        _options: Option<&AccessOptions>,
    ) -> Result<Vec<TenantInfo>, TenantResolverError> {
        // Return only the caller's tenant if it matches filter
        if Self::matches_filter(filter) {
            let tenant_id = ctx.tenant_id();
            Ok(vec![TenantInfo {
                id: tenant_id,
                name: format!("Tenant {tenant_id}"),
                status: TenantStatus::Active,
                tenant_type: None,
            }])
        } else {
            Ok(vec![])
        }
    }
}
