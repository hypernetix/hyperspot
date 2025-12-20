//! Domain service for the Types Registry module.

use std::sync::Arc;

use types_registry_sdk::{GtsEntity, ListQuery, RegisterResult};

use super::error::DomainError;
use super::repo::GtsRepository;

/// Domain service for GTS entity operations.
///
/// This service orchestrates business logic and delegates storage
/// operations to the repository.
pub struct TypesRegistryService {
    repo: Arc<dyn GtsRepository>,
}

impl TypesRegistryService {
    /// Creates a new `TypesRegistryService` with the given repository.
    #[must_use]
    pub fn new(repo: Arc<dyn GtsRepository>) -> Self {
        Self { repo }
    }

    /// Registers GTS entities in batch.
    ///
    /// Returns a `RegisterResult` for each input entity, preserving order.
    #[must_use]
    pub fn register(&self, entities: Vec<serde_json::Value>) -> Vec<RegisterResult> {
        let is_production = self.repo.is_production();
        let mut results = Vec::with_capacity(entities.len());

        for entity in entities {
            let gts_id = Self::extract_gts_id(&entity);
            let result = match self.repo.register(&entity, is_production) {
                Ok(registered) => RegisterResult::Ok(registered),
                Err(e) => RegisterResult::Err {
                    gts_id,
                    error: e.into(),
                },
            };
            results.push(result);
        }

        results
    }

    /// Retrieves a single GTS entity by its identifier.
    pub fn get(&self, gts_id: &str) -> Result<GtsEntity, DomainError> {
        self.repo.get(gts_id)
    }

    /// Lists GTS entities matching the given query.
    pub fn list(&self, query: &ListQuery) -> Result<Vec<GtsEntity>, DomainError> {
        self.repo.list(query)
    }

    /// Switches the registry from configuration mode to production mode.
    ///
    /// This validates all entities in temporary storage and moves them
    /// to persistent storage if validation succeeds.
    ///
    /// # Errors
    ///
    /// Returns `ProductionCommitFailed` with typed `ValidationError` structs
    /// containing the GTS ID and error message for each failing entity.
    pub fn switch_to_production(&self) -> Result<(), DomainError> {
        use crate::domain::error::ValidationError;
        self.repo.switch_to_production().map_err(|errors| {
            let typed_errors: Vec<ValidationError> = errors
                .into_iter()
                .map(|s| ValidationError::from_string(&s))
                .collect();
            DomainError::ProductionCommitFailed(typed_errors)
        })
    }

    /// Returns whether the registry is in production mode.
    #[must_use]
    pub fn is_production(&self) -> bool {
        self.repo.is_production()
    }

    /// Extracts the GTS ID from an entity JSON value.
    fn extract_gts_id(entity: &serde_json::Value) -> Option<String> {
        if let Some(obj) = entity.as_object() {
            for field in &["$id", "gtsId", "id"] {
                if let Some(id) = obj.get(*field).and_then(|v| v.as_str()) {
                    return Some(id.to_owned());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, Ordering};
    use types_registry_sdk::GtsEntityKind;
    use uuid::Uuid;

    struct MockRepo {
        is_production: AtomicBool,
        fail_switch: bool,
    }

    impl MockRepo {
        fn new() -> Self {
            Self {
                is_production: AtomicBool::new(false),
                fail_switch: false,
            }
        }

        fn with_fail_switch() -> Self {
            Self {
                is_production: AtomicBool::new(false),
                fail_switch: true,
            }
        }
    }

    impl GtsRepository for MockRepo {
        fn register(
            &self,
            entity: &serde_json::Value,
            _validate: bool,
        ) -> Result<GtsEntity, DomainError> {
            let gts_id = entity
                .get("$id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DomainError::invalid_gts_id("No $id field"))?;

            if gts_id.contains("fail") {
                return Err(DomainError::validation_failed("Test failure"));
            }

            Ok(GtsEntity::new(
                Uuid::nil(),
                gts_id.to_owned(),
                vec![],
                GtsEntityKind::Type,
                entity.clone(),
                None,
            ))
        }

        fn get(&self, gts_id: &str) -> Result<GtsEntity, DomainError> {
            if gts_id.contains("notfound") {
                return Err(DomainError::not_found(gts_id));
            }
            Ok(GtsEntity::new(
                Uuid::nil(),
                gts_id.to_owned(),
                vec![],
                GtsEntityKind::Type,
                json!({}),
                None,
            ))
        }

        fn list(&self, _query: &ListQuery) -> Result<Vec<GtsEntity>, DomainError> {
            Ok(vec![GtsEntity::new(
                Uuid::nil(),
                "gts.test.pkg.ns.type.v1~".to_owned(),
                vec![],
                GtsEntityKind::Type,
                json!({}),
                None,
            )])
        }

        fn exists(&self, _gts_id: &str) -> bool {
            true
        }

        fn is_production(&self) -> bool {
            self.is_production.load(Ordering::SeqCst)
        }

        fn switch_to_production(&self) -> Result<(), Vec<String>> {
            if self.fail_switch {
                // Return errors in "gts_id: message" format for ValidationError::from_string
                return Err(vec![
                    "gts.test1~: error1".to_owned(),
                    "gts.test2~: error2".to_owned(),
                ]);
            }
            self.is_production.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_extract_gts_id() {
        let _service = TypesRegistryService::new(Arc::new(MockRepo::new()));

        let entity = json!({"$id": "gts.acme.core.events.test.v1~"});
        assert_eq!(
            TypesRegistryService::extract_gts_id(&entity),
            Some("gts.acme.core.events.test.v1~".to_owned())
        );

        let entity = json!({"gtsId": "gts.acme.core.events.test.v1~"});
        assert_eq!(
            TypesRegistryService::extract_gts_id(&entity),
            Some("gts.acme.core.events.test.v1~".to_owned())
        );

        let entity = json!({"id": "gts.acme.core.events.test.v1~"});
        assert_eq!(
            TypesRegistryService::extract_gts_id(&entity),
            Some("gts.acme.core.events.test.v1~".to_owned())
        );

        let entity = json!({"other": "value"});
        assert_eq!(TypesRegistryService::extract_gts_id(&entity), None);

        let entity = json!("not an object");
        assert_eq!(TypesRegistryService::extract_gts_id(&entity), None);
    }

    #[test]
    fn test_register_success() {
        let service = TypesRegistryService::new(Arc::new(MockRepo::new()));

        let entities = vec![
            json!({"$id": "gts.acme.core.events.test.v1~"}),
            json!({"$id": "gts.acme.core.events.test2.v1~"}),
        ];

        let results = service.register(entities);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[test]
    fn test_register_with_failures() {
        let service = TypesRegistryService::new(Arc::new(MockRepo::new()));

        let entities = vec![
            json!({"$id": "gts.acme.core.events.test.v1~"}),
            json!({"$id": "gts.acme.core.events.fail.v1~"}),
            json!({"other": "no id"}),
        ];

        let results = service.register(entities);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_err());
    }

    #[test]
    fn test_get_success() {
        let service = TypesRegistryService::new(Arc::new(MockRepo::new()));
        let result = service.get("gts.acme.core.events.test.v1~");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_not_found() {
        let service = TypesRegistryService::new(Arc::new(MockRepo::new()));
        let result = service.get("gts.notfound.pkg.ns.type.v1~");
        assert!(result.is_err());
    }

    #[test]
    fn test_list() {
        let service = TypesRegistryService::new(Arc::new(MockRepo::new()));
        let result = service.list(&ListQuery::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_switch_to_production_success() {
        let service = TypesRegistryService::new(Arc::new(MockRepo::new()));
        assert!(!service.is_production());

        let result = service.switch_to_production();
        assert!(result.is_ok());
        assert!(service.is_production());
    }

    #[test]
    fn test_switch_to_production_failure() {
        let service = TypesRegistryService::new(Arc::new(MockRepo::with_fail_switch()));
        let result = service.switch_to_production();
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ProductionCommitFailed(errors) => {
                assert_eq!(errors.len(), 2);
                assert_eq!(errors[0].gts_id, "gts.test1~");
                assert_eq!(errors[0].message, "error1");
                assert_eq!(errors[1].gts_id, "gts.test2~");
                assert_eq!(errors[1].message, "error2");
            }
            _ => panic!("Expected ProductionCommitFailed"),
        }
    }

    #[test]
    fn test_is_production() {
        let service = TypesRegistryService::new(Arc::new(MockRepo::new()));
        assert!(!service.is_production());
    }
}
