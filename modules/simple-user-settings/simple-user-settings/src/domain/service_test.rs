#[cfg(test)]
mod tests {
    use super::super::*;
    use async_trait::async_trait;
    use modkit_security::SecurityContext;
    use simple_user_settings_sdk::models::{SimpleUserSettings, SimpleUserSettingsPatch};
    use std::sync::Arc;
    use uuid::Uuid;

    // Mock repository for testing
    struct MockRepository {
        find_result: Option<SimpleUserSettings>,
        upsert_result: SimpleUserSettings,
    }

    #[async_trait]
    impl repo::SettingsRepository for MockRepository {
        async fn find_by_user(
            &self,
            _ctx: &SecurityContext,
        ) -> anyhow::Result<Option<SimpleUserSettings>> {
            Ok(self.find_result.clone())
        }

        async fn upsert_full(
            &self,
            _ctx: &SecurityContext,
            theme: Option<String>,
            language: Option<String>,
        ) -> anyhow::Result<SimpleUserSettings> {
            Ok(SimpleUserSettings {
                user_id: self.upsert_result.user_id,
                tenant_id: self.upsert_result.tenant_id,
                theme,
                language,
            })
        }

        async fn upsert_patch(
            &self,
            _ctx: &SecurityContext,
            patch: SimpleUserSettingsPatch,
        ) -> anyhow::Result<SimpleUserSettings> {
            let mut result = self.upsert_result.clone();
            if let Some(theme) = patch.theme {
                result.theme = Some(theme);
            }
            if let Some(language) = patch.language {
                result.language = Some(language);
            }
            Ok(result)
        }
    }

    fn create_test_context() -> SecurityContext {
        SecurityContext::anonymous()
    }

    #[tokio::test]
    async fn test_get_settings_returns_existing() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let existing = SimpleUserSettings {
            user_id,
            tenant_id,
            theme: Some("dark".to_owned()),
            language: Some("en".to_owned()),
        };

        let repo = Arc::new(MockRepository {
            find_result: Some(existing.clone()),
            upsert_result: existing.clone(),
        });

        let service = service::Service::new(repo, service::ServiceConfig::default());
        let ctx = create_test_context();

        let result = service.get_settings(&ctx).await.unwrap();

        assert_eq!(result.theme, Some("dark".to_owned()));
        assert_eq!(result.language, Some("en".to_owned()));
    }

    #[tokio::test]
    async fn test_get_settings_returns_defaults_when_not_found() {
        let repo = Arc::new(MockRepository {
            find_result: None,
            upsert_result: SimpleUserSettings {
                user_id: Uuid::default(),
                tenant_id: Uuid::default(),
                theme: None,
                language: None,
            },
        });

        let service = service::Service::new(repo, service::ServiceConfig::default());
        let ctx = create_test_context();

        let result = service.get_settings(&ctx).await.unwrap();

        assert_eq!(result.user_id, ctx.subject_id());
        assert_eq!(result.tenant_id, ctx.tenant_id());
        assert_eq!(result.theme, None);
        assert_eq!(result.language, None);
    }

    #[tokio::test]
    async fn test_update_settings_success() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let repo = Arc::new(MockRepository {
            find_result: None,
            upsert_result: SimpleUserSettings {
                user_id,
                tenant_id,
                theme: None,
                language: None,
            },
        });

        let service = service::Service::new(repo, service::ServiceConfig::default());
        let ctx = create_test_context();

        let result = service
            .update_settings(&ctx, Some("light".to_owned()), Some("es".to_owned()))
            .await
            .unwrap();

        assert_eq!(result.theme, Some("light".to_owned()));
        assert_eq!(result.language, Some("es".to_owned()));
    }

    #[tokio::test]
    async fn test_update_settings_validates_max_length() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let repo = Arc::new(MockRepository {
            find_result: None,
            upsert_result: SimpleUserSettings {
                user_id,
                tenant_id,
                theme: None,
                language: None,
            },
        });

        let service = service::Service::new(
            repo,
            service::ServiceConfig {
                max_field_length: 10,
            },
        );
        let ctx = create_test_context();

        let too_long = "a".repeat(11);
        let result = service
            .update_settings(&ctx, Some(too_long), Some("en".to_owned()))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, error::DomainError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_patch_settings_updates_only_provided_fields() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let repo = Arc::new(MockRepository {
            find_result: Some(SimpleUserSettings {
                user_id,
                tenant_id,
                theme: Some("dark".to_owned()),
                language: Some("en".to_owned()),
            }),
            upsert_result: SimpleUserSettings {
                user_id,
                tenant_id,
                theme: Some("dark".to_owned()),
                language: Some("en".to_owned()),
            },
        });

        let service = service::Service::new(repo, service::ServiceConfig::default());
        let ctx = create_test_context();

        // Only update theme
        let patch = SimpleUserSettingsPatch {
            theme: Some("light".to_owned()),
            language: None,
        };

        let result = service.patch_settings(&ctx, patch).await.unwrap();

        assert_eq!(result.theme, Some("light".to_owned()));
        assert_eq!(result.language, Some("en".to_owned())); // Should remain unchanged
    }

    #[tokio::test]
    async fn test_patch_settings_validates_max_length() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let repo = Arc::new(MockRepository {
            find_result: None,
            upsert_result: SimpleUserSettings {
                user_id,
                tenant_id,
                theme: None,
                language: None,
            },
        });

        let service = service::Service::new(
            repo,
            service::ServiceConfig {
                max_field_length: 10,
            },
        );
        let ctx = create_test_context();

        let too_long = "a".repeat(11);
        let patch = SimpleUserSettingsPatch {
            theme: None,
            language: Some(too_long),
        };

        let result = service.patch_settings(&ctx, patch).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, error::DomainError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_patch_settings_empty_patch_succeeds() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let existing = SimpleUserSettings {
            user_id,
            tenant_id,
            theme: Some("dark".to_owned()),
            language: Some("en".to_owned()),
        };

        let repo = Arc::new(MockRepository {
            find_result: Some(existing.clone()),
            upsert_result: existing.clone(),
        });

        let service = service::Service::new(repo, service::ServiceConfig::default());
        let ctx = create_test_context();

        // Empty patch - no fields to update
        let patch = SimpleUserSettingsPatch {
            theme: None,
            language: None,
        };

        let result = service.patch_settings(&ctx, patch).await.unwrap();

        // Should return existing values unchanged
        assert_eq!(result.theme, Some("dark".to_owned()));
        assert_eq!(result.language, Some("en".to_owned()));
    }
}
