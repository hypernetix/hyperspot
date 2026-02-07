//! Integration tests for the Settings service.
//!
//! These tests use an in-memory `SQLite` database since `DBRunner` is a sealed trait
//! and cannot be mocked. All tests use real database operations.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use modkit_db::migration_runner::run_migrations_for_testing;
    use modkit_db::{ConnectOpts, DBProvider, Db, connect_db};
    use modkit_security::SecurityContext;
    use simple_user_settings_sdk::models::{SimpleUserSettingsPatch, SimpleUserSettingsUpdate};
    use uuid::Uuid;

    use crate::domain::error::DomainError;
    use crate::domain::service::{Service, ServiceConfig};
    use crate::infra::storage::migrations::Migrator;
    use crate::infra::storage::sea_orm_repo::SeaOrmSettingsRepository;

    type ConcreteService = Service<SeaOrmSettingsRepository>;

    /// Create an in-memory database with migrations applied.
    async fn inmem_db() -> Db {
        use sea_orm_migration::MigratorTrait;

        let opts = ConnectOpts {
            max_conns: Some(1),
            min_conns: Some(1),
            ..Default::default()
        };
        let db = connect_db("sqlite::memory:", opts)
            .await
            .expect("Failed to connect to in-memory database");

        run_migrations_for_testing(&db, Migrator::migrations())
            .await
            .expect("Failed to run migrations");

        db
    }

    fn create_test_context() -> SecurityContext {
        SecurityContext::builder()
            .tenant_id(Uuid::new_v4())
            .subject_id(Uuid::new_v4())
            .build()
    }

    fn build_service(db: Db, config: ServiceConfig) -> ConcreteService {
        let repo = Arc::new(SeaOrmSettingsRepository::new());
        let db: Arc<DBProvider<modkit_db::DbError>> = Arc::new(DBProvider::new(db));
        Service::new(db, repo, config)
    }

    // =========================================================================
    // get_settings tests
    // =========================================================================

    #[tokio::test]
    async fn test_get_settings_returns_defaults_when_not_found() {
        let db = inmem_db().await;
        let service = build_service(db, ServiceConfig::default());
        let ctx = create_test_context();

        let result = service.get_settings(&ctx).await.unwrap();

        assert_eq!(result.user_id, ctx.subject_id());
        assert_eq!(result.tenant_id, ctx.tenant_id());
        assert_eq!(result.theme, None);
        assert_eq!(result.language, None);
    }

    #[tokio::test]
    async fn test_get_settings_returns_existing() {
        let db = inmem_db().await;
        let service = build_service(db, ServiceConfig::default());
        let ctx = create_test_context();

        // First, create settings
        let _ = service
            .update_settings(
                &ctx,
                SimpleUserSettingsUpdate {
                    theme: "dark".to_owned(),
                    language: "en".to_owned(),
                },
            )
            .await
            .unwrap();

        // Then retrieve them
        let result = service.get_settings(&ctx).await.unwrap();

        assert_eq!(result.theme, Some("dark".to_owned()));
        assert_eq!(result.language, Some("en".to_owned()));
    }

    // =========================================================================
    // update_settings tests
    // =========================================================================

    #[tokio::test]
    async fn test_update_settings_success() {
        let db = inmem_db().await;
        let service = build_service(db, ServiceConfig::default());
        let ctx = create_test_context();

        let result = service
            .update_settings(
                &ctx,
                SimpleUserSettingsUpdate {
                    theme: "light".to_owned(),
                    language: "es".to_owned(),
                },
            )
            .await
            .unwrap();

        assert_eq!(result.theme, Some("light".to_owned()));
        assert_eq!(result.language, Some("es".to_owned()));
        assert_eq!(result.user_id, ctx.subject_id());
        assert_eq!(result.tenant_id, ctx.tenant_id());
    }

    #[tokio::test]
    async fn test_update_settings_validates_max_length_for_theme() {
        let db = inmem_db().await;
        let service = build_service(
            db,
            ServiceConfig {
                max_field_length: 10,
            },
        );
        let ctx = create_test_context();

        let too_long = "a".repeat(11);
        let result = service
            .update_settings(
                &ctx,
                SimpleUserSettingsUpdate {
                    theme: too_long,
                    language: "en".to_owned(),
                },
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, DomainError::Validation { field, .. } if field == "theme"));
    }

    #[tokio::test]
    async fn test_update_settings_validates_max_length_for_language() {
        let db = inmem_db().await;
        let service = build_service(
            db,
            ServiceConfig {
                max_field_length: 10,
            },
        );
        let ctx = create_test_context();

        let too_long = "a".repeat(11);
        let result = service
            .update_settings(
                &ctx,
                SimpleUserSettingsUpdate {
                    theme: "dark".to_owned(),
                    language: too_long,
                },
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, DomainError::Validation { field, .. } if field == "language"));
    }

    // =========================================================================
    // patch_settings tests
    // =========================================================================

    #[tokio::test]
    async fn test_patch_settings_updates_only_provided_fields() {
        let db = inmem_db().await;
        let service = build_service(db, ServiceConfig::default());
        let ctx = create_test_context();

        // First create initial settings
        let _ = service
            .update_settings(
                &ctx,
                SimpleUserSettingsUpdate {
                    theme: "dark".to_owned(),
                    language: "en".to_owned(),
                },
            )
            .await
            .unwrap();

        // Patch only theme
        let result = service
            .patch_settings(
                &ctx,
                SimpleUserSettingsPatch {
                    theme: Some("light".to_owned()),
                    language: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(result.theme, Some("light".to_owned()));
        assert_eq!(result.language, Some("en".to_owned())); // Should remain unchanged
    }

    #[tokio::test]
    async fn test_patch_settings_validates_max_length() {
        let db = inmem_db().await;
        let service = build_service(
            db,
            ServiceConfig {
                max_field_length: 10,
            },
        );
        let ctx = create_test_context();

        let too_long = "a".repeat(11);
        let result = service
            .patch_settings(
                &ctx,
                SimpleUserSettingsPatch {
                    theme: None,
                    language: Some(too_long),
                },
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, DomainError::Validation { field, .. } if field == "language"));
    }

    #[tokio::test]
    async fn test_patch_settings_empty_patch_returns_existing() {
        let db = inmem_db().await;
        let service = build_service(db, ServiceConfig::default());
        let ctx = create_test_context();

        // First create settings
        let _ = service
            .update_settings(
                &ctx,
                SimpleUserSettingsUpdate {
                    theme: "dark".to_owned(),
                    language: "en".to_owned(),
                },
            )
            .await
            .unwrap();

        // Empty patch - no fields to update
        let result = service
            .patch_settings(
                &ctx,
                SimpleUserSettingsPatch {
                    theme: None,
                    language: None,
                },
            )
            .await
            .unwrap();

        // Should return existing values unchanged
        assert_eq!(result.theme, Some("dark".to_owned()));
        assert_eq!(result.language, Some("en".to_owned()));
    }

    #[tokio::test]
    async fn test_patch_settings_creates_if_not_exists() {
        let db = inmem_db().await;
        let service = build_service(db, ServiceConfig::default());
        let ctx = create_test_context();

        // Patch without existing settings
        let result = service
            .patch_settings(
                &ctx,
                SimpleUserSettingsPatch {
                    theme: Some("dark".to_owned()),
                    language: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(result.theme, Some("dark".to_owned()));
        assert_eq!(result.language, None);
    }

    // =========================================================================
    // Tenant isolation tests
    // =========================================================================

    #[tokio::test]
    async fn test_settings_isolated_by_user() {
        let db = inmem_db().await;
        let service = build_service(db, ServiceConfig::default());

        let tenant_id = Uuid::new_v4();
        let user1 = SecurityContext::builder()
            .tenant_id(tenant_id)
            .subject_id(Uuid::new_v4())
            .build();
        let user2 = SecurityContext::builder()
            .tenant_id(tenant_id)
            .subject_id(Uuid::new_v4())
            .build();

        // User 1 creates settings
        let _ = service
            .update_settings(
                &user1,
                SimpleUserSettingsUpdate {
                    theme: "dark".to_owned(),
                    language: "en".to_owned(),
                },
            )
            .await
            .unwrap();

        // User 2 should get default settings
        let result = service.get_settings(&user2).await.unwrap();
        assert_eq!(result.theme, None);
        assert_eq!(result.language, None);
        assert_eq!(result.user_id, user2.subject_id());
    }

    #[tokio::test]
    async fn test_settings_isolated_by_tenant() {
        let db = inmem_db().await;
        let service = build_service(db, ServiceConfig::default());

        let user_id = Uuid::new_v4();
        let tenant1 = SecurityContext::builder()
            .tenant_id(Uuid::new_v4())
            .subject_id(user_id)
            .build();
        let tenant2 = SecurityContext::builder()
            .tenant_id(Uuid::new_v4())
            .subject_id(user_id)
            .build();

        // Same user in tenant 1 creates settings
        let _ = service
            .update_settings(
                &tenant1,
                SimpleUserSettingsUpdate {
                    theme: "dark".to_owned(),
                    language: "en".to_owned(),
                },
            )
            .await
            .unwrap();

        // Same user in tenant 2 should get default settings
        let result = service.get_settings(&tenant2).await.unwrap();
        assert_eq!(result.theme, None);
        assert_eq!(result.language, None);
        assert_eq!(result.tenant_id, tenant2.tenant_id());
    }
}
