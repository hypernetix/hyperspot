#![cfg(feature = "integration")]

mod common;
use anyhow::Result;
use chrono::Utc;

#[tokio::test]
async fn users_info_works_with_postgres() -> Result<()> {
    let dut = common::bring_up_postgres().await?;

    // Connect via modkit-db
    let db = modkit_db::DbHandle::connect(&dut.url, modkit_db::ConnectOpts::default()).await?;

    // Apply migrations using SeaORM migrator
    let db_conn = db.sea();
    use sea_orm_migration::MigratorTrait;
    users_info::infra::storage::migrations::Migrator::up(&db_conn, None)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

    // Test basic CRUD operations using the repository
    test_repository_operations(&db_conn).await?;

    // Test domain service operations
    test_service_operations(&db_conn).await?;

    Ok(())
}

async fn test_repository_operations(db_conn: &sea_orm::DatabaseConnection) -> Result<()> {
    use users_info::contract::model::User;
    use users_info::domain::repo::UsersRepository;
    use users_info::infra::storage::sea_orm_repo::SeaOrmUsersRepository;
    use uuid::Uuid;

    let repo = SeaOrmUsersRepository::new(db_conn.clone());

    // Create a user manually using the repository interface
    let now = Utc::now();
    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        email: "test@example.com".to_string(),
        display_name: "Test User".to_string(),
        created_at: now,
        updated_at: now,
    };

    // Test insert user
    repo.insert(user.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert user: {}", e))?;

    // Test find by ID
    let found_user = repo
        .find_by_id(user.id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to find user: {}", e))?;

    assert!(found_user.is_some());
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.email, user.email);

    // Test email exists
    let email_exists = repo
        .email_exists(&user.email)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to check email exists: {}", e))?;

    assert!(email_exists);

    // Test update user
    let mut updated_user = found_user.clone();
    updated_user.display_name = "Updated Name".to_string();
    updated_user.updated_at = Utc::now();

    repo.update(updated_user.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to update user: {}", e))?;

    // Verify update
    let found_updated = repo
        .find_by_id(user.id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to find updated user: {}", e))?;

    assert!(found_updated.is_some());
    let found_updated = found_updated.unwrap();
    assert_eq!(found_updated.display_name, "Updated Name");

    // Test list users with pagination
    let query = modkit_odata::ODataQuery::default();
    let page = repo
        .list_users_page(&query)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list users: {}", e))?;

    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, user.id);

    // Test delete user
    let deleted = repo
        .delete(user.id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to delete user: {}", e))?;

    assert!(deleted);

    // Verify user is deleted
    let not_found = repo
        .find_by_id(user.id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to check deleted user: {}", e))?;

    assert!(not_found.is_none());

    Ok(())
}

async fn test_service_operations(db_conn: &sea_orm::DatabaseConnection) -> Result<()> {
    use std::sync::Arc;
    use users_info::contract::model::NewUser;
    use users_info::domain::error::DomainError;
    use users_info::domain::events::UserDomainEvent;
    use users_info::domain::ports::{AuditPort, EventPublisher};
    use users_info::domain::service::{Service, ServiceConfig};
    use users_info::infra::storage::sea_orm_repo::SeaOrmUsersRepository;
    use uuid::Uuid;

    // Mock audit port for tests - always succeeds
    struct MockAuditPort;

    #[async_trait::async_trait]
    impl AuditPort for MockAuditPort {
        async fn get_user_access(&self, _id: Uuid) -> Result<(), DomainError> {
            Ok(())
        }

        async fn notify_user_created(&self) -> Result<(), DomainError> {
            Ok(())
        }
    }

    // Mock event publisher for tests - just ignores events
    struct MockEventPublisher;

    impl EventPublisher<UserDomainEvent> for MockEventPublisher {
        fn publish(&self, _event: &UserDomainEvent) {
            // no-op in tests
        }
    }

    let repo = Arc::new(SeaOrmUsersRepository::new(db_conn.clone()));
    let events: Arc<dyn EventPublisher<UserDomainEvent>> = Arc::new(MockEventPublisher);
    let audit: Arc<dyn AuditPort> = Arc::new(MockAuditPort);
    let config = ServiceConfig::default();
    let service = Service::new(repo, events, audit, config);

    // Test create user through service
    let new_user = NewUser {
        email: "service@example.com".to_string(),
        display_name: "Service User".to_string(),
    };

    let user = service
        .create_user(new_user)
        .await
        .map_err(|e| anyhow::anyhow!("Service failed to create user: {}", e))?;

    assert_eq!(user.email, "service@example.com");
    assert_eq!(user.display_name, "Service User");

    // Test get user through service
    let found_user = service
        .get_user(user.id)
        .await
        .map_err(|e| anyhow::anyhow!("Service failed to get user: {}", e))?;

    assert_eq!(found_user.id, user.id);

    // Test list users through service
    let query = modkit_odata::ODataQuery::default();
    let page = service
        .list_users_page(query)
        .await
        .map_err(|e| anyhow::anyhow!("Service failed to list users: {}", e))?;

    assert!(!page.items.is_empty());

    // Test update through service
    use users_info::contract::model::UserPatch;
    let patch = UserPatch {
        display_name: Some("Updated Service User".to_string()),
        ..Default::default()
    };

    let updated = service
        .update_user(user.id, patch)
        .await
        .map_err(|e| anyhow::anyhow!("Service failed to update user: {}", e))?;

    assert_eq!(updated.display_name, "Updated Service User");

    // Test delete through service
    service
        .delete_user(user.id)
        .await
        .map_err(|e| anyhow::anyhow!("Service failed to delete user: {}", e))?;

    // Verify deletion
    let result = service.get_user(user.id).await;
    assert!(result.is_err()); // Should return NotFound error

    Ok(())
}
