#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test support utilities for `users_info` integration tests.
//!
//! Provides helper functions for creating security contexts, test databases,
//! and seeding test data.

#![allow(dead_code)] // Support module provides utilities that may not all be used

use modkit_db::secure::SecureConn;
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
pub fn ctx_allow_tenants(tenants: &[Uuid]) -> SecurityContext {
    // Use the first tenant as the context tenant_id
    let tenant_id = tenants.first().copied().unwrap_or_else(Uuid::new_v4);
    SecurityContext::builder()
        .tenant_id(tenant_id)
        .subject_id(Uuid::new_v4())
        .build()
}

/// Create a security context that allows access to specific resources.
///
/// Uses a random subject ID for testing purposes.
#[must_use]
pub fn ctx_allow_resources(_resources: &[Uuid]) -> SecurityContext {
    SecurityContext::builder()
        .tenant_id(Uuid::new_v4())
        .subject_id(Uuid::new_v4())
        .build()
}

/// Create a security context with a specific subject ID and tenant access.
///
/// Useful when you need to test `owner_id` or subject-specific behavior.
#[must_use]
pub fn ctx_with_subject(subject_id: Uuid, tenants: &[Uuid]) -> SecurityContext {
    let tenant_id = tenants.first().copied().unwrap_or_else(Uuid::new_v4);
    SecurityContext::builder()
        .tenant_id(tenant_id)
        .subject_id(subject_id)
        .build()
}

/// Create a deny-all security context.
///
/// This context will deny access to all data (empty scope).
#[must_use]
pub fn ctx_deny_all() -> SecurityContext {
    let subject = Subject::new(Uuid::new_v4());
    SecurityContext::new(AccessScope::default(), subject)
}

/// Create a root security context (system-level access).
///
/// This context bypasses all tenant filtering and allows access to all data.
#[must_use]
pub fn ctx_root() -> SecurityContext {
    SecurityContext::root()
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
