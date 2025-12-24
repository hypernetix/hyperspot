//! In-memory repository implementation using gts-rust.

use std::sync::atomic::{AtomicBool, Ordering};

use gts::{GtsConfig, GtsID, GtsIdSegment, GtsOps, GtsWildcard};
use parking_lot::Mutex;
use types_registry_sdk::{GtsEntity, ListQuery, SegmentMatchScope};

use crate::domain::error::DomainError;
use crate::domain::repo::GtsRepository;

/// In-memory repository for GTS entities using gts-rust.
///
/// Implements two-phase storage:
/// - **Configuration phase**: Entities stored in `temporary` without validation
/// - **Ready phase**: Entities validated and stored in `persistent`
///
/// Note: Uses `Mutex` instead of `RwLock` because `GtsOps` contains a
/// `Box<dyn GtsReader>` which is not `Sync`.
pub struct InMemoryGtsRepository {
    /// Temporary storage during configuration phase.
    temporary: Mutex<GtsOps>,
    /// Persistent storage after ready commit.
    persistent: Mutex<GtsOps>,
    /// Flag indicating ready mode.
    is_ready: AtomicBool,
    /// GTS configuration.
    config: GtsConfig,
}

impl InMemoryGtsRepository {
    /// Creates a new in-memory repository with the given GTS configuration.
    #[must_use]
    pub fn new(config: GtsConfig) -> Self {
        Self {
            temporary: Mutex::new(GtsOps::new(None, None, 0)),
            persistent: Mutex::new(GtsOps::new(None, None, 0)),
            is_ready: AtomicBool::new(false),
            config,
        }
    }

    /// Converts a gts-rust entity result to our SDK `GtsEntity`.
    fn to_gts_entity(gts_id: &str, content: &serde_json::Value) -> Result<GtsEntity, DomainError> {
        let parsed = GtsID::new(gts_id).map_err(|e| DomainError::invalid_gts_id(e.to_string()))?;

        let segments: Vec<GtsIdSegment> = parsed.gts_id_segments.clone();

        let is_schema = gts_id.ends_with('~');

        let id = parsed.to_uuid();

        let description = content
            .get("description")
            .and_then(|v| v.as_str())
            .map(ToOwned::to_owned);

        Ok(GtsEntity::new(
            id,
            gts_id.to_owned(),
            segments,
            is_schema,
            content.clone(),
            description,
        ))
    }

    /// Extracts the GTS ID from an entity JSON value using configured fields.
    ///
    /// Strips the `gts://` URI prefix from `$id` fields for JSON Schema compatibility (gts-rust v0.7.0+).
    fn extract_gts_id(&self, entity: &serde_json::Value) -> Option<String> {
        if let Some(obj) = entity.as_object() {
            for field in &self.config.entity_id_fields {
                if let Some(id) = obj.get(field).and_then(|v| v.as_str()) {
                    // Strip gts:// prefix from $id field (JSON Schema URI format)
                    let cleaned_id = if field == "$id" {
                        id.strip_prefix("gts://").unwrap_or(id)
                    } else {
                        id
                    };
                    return Some(cleaned_id.to_owned());
                }
            }
        }
        None
    }

    /// Checks if an entity matches the given query filters.
    fn matches_query(entity: &GtsEntity, query: &ListQuery) -> bool {
        if let Some(ref pattern) = query.pattern {
            if let Ok(wildcard) = GtsWildcard::new(pattern) {
                if let Ok(gts_id) = GtsID::new(&entity.gts_id) {
                    if !gts_id.wildcard_match(&wildcard) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        if let Some(is_type) = query.is_type {
            if entity.is_type() != is_type {
                return false;
            }
        }

        let segments_to_check: Vec<&GtsIdSegment> = match query.segment_scope {
            SegmentMatchScope::Primary => entity.segments.first().into_iter().collect(),
            SegmentMatchScope::Any => entity.segments.iter().collect(),
        };

        if let Some(ref vendor) = query.vendor {
            if !segments_to_check.iter().any(|s| s.vendor == *vendor) {
                return false;
            }
        }

        if let Some(ref package) = query.package {
            if !segments_to_check.iter().any(|s| s.package == *package) {
                return false;
            }
        }

        if let Some(ref namespace) = query.namespace {
            if !segments_to_check.iter().any(|s| s.namespace == *namespace) {
                return false;
            }
        }

        true
    }
}

impl GtsRepository for InMemoryGtsRepository {
    fn register(
        &self,
        entity: &serde_json::Value,
        validate: bool,
    ) -> Result<GtsEntity, DomainError> {
        let gts_id = self
            .extract_gts_id(entity)
            .ok_or_else(|| DomainError::invalid_gts_id("No GTS ID field found in entity"))?;

        GtsID::new(&gts_id).map_err(|e| DomainError::invalid_gts_id(e.to_string()))?;

        if self.is_ready.load(Ordering::SeqCst) {
            let mut persistent = self.persistent.lock();

            if let Some(existing) = persistent.store.get(&gts_id) {
                if existing.content == *entity {
                    return Self::to_gts_entity(&gts_id, entity);
                }
                return Err(DomainError::already_exists(&gts_id));
            }

            let result = persistent.add_entity(entity, validate);
            if !result.ok {
                return Err(DomainError::validation_failed(result.error));
            }

            Self::to_gts_entity(&gts_id, entity)
        } else {
            let mut temporary = self.temporary.lock();

            if let Some(existing) = temporary.store.get(&gts_id) {
                if existing.content == *entity {
                    return Self::to_gts_entity(&gts_id, entity);
                }
                return Err(DomainError::already_exists(&gts_id));
            }

            let result = temporary.add_entity(entity, false);
            if !result.ok {
                return Err(DomainError::validation_failed(result.error));
            }

            Self::to_gts_entity(&gts_id, entity)
        }
    }

    fn get(&self, gts_id: &str) -> Result<GtsEntity, DomainError> {
        let mut persistent = self.persistent.lock();

        if let Some(entity) = persistent.store.get(gts_id) {
            return Self::to_gts_entity(gts_id, &entity.content);
        }

        Err(DomainError::not_found(gts_id))
    }

    fn list(&self, query: &ListQuery) -> Result<Vec<GtsEntity>, DomainError> {
        let persistent = self.persistent.lock();
        let mut results = Vec::new();

        for (gts_id, gts_entity) in persistent.store.items() {
            if let Ok(entity) = Self::to_gts_entity(gts_id, &gts_entity.content) {
                if Self::matches_query(&entity, query) {
                    results.push(entity);
                }
            }
        }

        Ok(results)
    }

    fn exists(&self, gts_id: &str) -> bool {
        let mut persistent = self.persistent.lock();
        persistent.store.get(gts_id).is_some()
    }

    fn is_ready(&self) -> bool {
        self.is_ready.load(Ordering::SeqCst)
    }

    fn switch_to_ready(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        let gts_ids: Vec<String> = {
            let temporary = self.temporary.lock();
            temporary.store.items().map(|(id, _)| id.clone()).collect()
        };

        {
            let mut temporary = self.temporary.lock();
            for gts_id in &gts_ids {
                let result = temporary.validate_entity(gts_id);
                if !result.ok {
                    errors.push(format!("{gts_id}: {}", result.error));
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        {
            let mut temporary = self.temporary.lock();
            let mut persistent = self.persistent.lock();

            for gts_id in &gts_ids {
                if let Some(entity) = temporary.store.get(gts_id) {
                    let content = entity.content.clone();
                    let result = persistent.add_entity(&content, true);
                    if !result.ok {
                        errors.push(format!("{gts_id}: {}", result.error));
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        self.is_ready.store(true, Ordering::SeqCst);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn default_config() -> GtsConfig {
        crate::config::TypesRegistryConfig::default().to_gts_config()
    }

    #[test]
    fn test_register_in_configuration_mode() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "userId": { "type": "string" }
            }
        });

        let result = repo.register(&entity, false);
        assert!(result.is_ok());

        let registered = result.unwrap();
        assert_eq!(registered.gts_id, "gts.acme.core.events.user_created.v1~");
        assert!(registered.is_type());
    }

    #[test]
    fn test_register_duplicate_identical_succeeds() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        let result1 = repo.register(&entity, false);
        assert!(result1.is_ok());

        let result2 = repo.register(&entity, false);
        assert!(result2.is_ok(), "Idempotent registration should succeed");
    }

    #[test]
    fn test_register_duplicate_different_content_fails() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity1 = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        let entity2 = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "description": "Different content"
        });

        let result1 = repo.register(&entity1, false);
        assert!(result1.is_ok());

        let result2 = repo.register(&entity2, false);
        assert!(matches!(result2, Err(DomainError::AlreadyExists(_))));
    }

    #[test]
    fn test_register_invalid_gts_id_fails() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "$id": "invalid-gts-id",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        let result = repo.register(&entity, false);
        assert!(matches!(result, Err(DomainError::InvalidGtsId(_))));
    }

    #[test]
    fn test_register_missing_gts_id_fails() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        let result = repo.register(&entity, false);
        assert!(matches!(result, Err(DomainError::InvalidGtsId(_))));
    }

    #[test]
    fn test_switch_to_ready() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "userId": { "type": "string" }
            }
        });

        repo.register(&entity, false).unwrap();

        assert!(!repo.is_ready());

        let result = repo.switch_to_ready();
        assert!(result.is_ok());
        assert!(repo.is_ready());

        let get_result = repo.get("gts.acme.core.events.user_created.v1~");
        assert!(get_result.is_ok());
    }

    #[test]
    fn test_list_with_filters() {
        let repo = InMemoryGtsRepository::new(default_config());

        let type1 = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });
        let type2 = json!({
            "$id": "gts://gts.globex.core.events.order_placed.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        repo.register(&type1, false).unwrap();
        repo.register(&type2, false).unwrap();
        repo.switch_to_ready().unwrap();

        let query = ListQuery::default().with_vendor("acme");
        let results = repo.list(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].vendor(), Some("acme"));

        let query = ListQuery::default();
        let results = repo.list(&query).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_get_not_found() {
        let repo = InMemoryGtsRepository::new(default_config());
        repo.switch_to_ready().unwrap();

        let result = repo.get("gts.unknown.pkg.ns.type.v1~");
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    }

    #[test]
    fn test_register_in_ready_mode() {
        let repo = InMemoryGtsRepository::new(default_config());
        repo.switch_to_ready().unwrap();

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        let result = repo.register(&entity, true);
        assert!(result.is_ok());

        let get_result = repo.get("gts.acme.core.events.user_created.v1~");
        assert!(get_result.is_ok());
    }

    #[test]
    fn test_register_duplicate_identical_in_ready_mode_succeeds() {
        let repo = InMemoryGtsRepository::new(default_config());
        repo.switch_to_ready().unwrap();

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        repo.register(&entity, true).unwrap();
        let result = repo.register(&entity, true);
        assert!(
            result.is_ok(),
            "Idempotent registration should succeed in ready mode"
        );
    }

    #[test]
    fn test_register_duplicate_different_content_in_ready_mode_fails() {
        let repo = InMemoryGtsRepository::new(default_config());
        repo.switch_to_ready().unwrap();

        let entity1 = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        let entity2 = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "description": "Different content"
        });

        repo.register(&entity1, true).unwrap();
        let result = repo.register(&entity2, true);
        assert!(matches!(result, Err(DomainError::AlreadyExists(_))));
    }

    #[test]
    fn test_exists() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        repo.register(&entity, false).unwrap();
        repo.switch_to_ready().unwrap();

        assert!(repo.exists("gts.acme.core.events.user_created.v1~"));
        assert!(!repo.exists("gts.unknown.pkg.ns.type.v1~"));
    }

    #[test]
    fn test_list_with_is_type_filter() {
        let repo = InMemoryGtsRepository::new(default_config());

        let type_entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        repo.register(&type_entity, false).unwrap();
        repo.switch_to_ready().unwrap();

        let query = ListQuery::default().with_is_type(true);
        let results = repo.list(&query).unwrap();
        assert_eq!(results.len(), 1);

        let query = ListQuery::default().with_is_type(false);
        let results = repo.list(&query).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_list_with_package_filter() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        repo.register(&entity, false).unwrap();
        repo.switch_to_ready().unwrap();

        let query = ListQuery::default().with_package("core");
        let results = repo.list(&query).unwrap();
        assert_eq!(results.len(), 1);

        let query = ListQuery::default().with_package("other");
        let results = repo.list(&query).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_list_with_namespace_filter() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        repo.register(&entity, false).unwrap();
        repo.switch_to_ready().unwrap();

        let query = ListQuery::default().with_namespace("events");
        let results = repo.list(&query).unwrap();
        assert_eq!(results.len(), 1);

        let query = ListQuery::default().with_namespace("other");
        let results = repo.list(&query).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_list_with_pattern_filter() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        repo.register(&entity, false).unwrap();
        repo.switch_to_ready().unwrap();

        let query = ListQuery::default().with_pattern("gts.acme.*");
        let results = repo.list(&query).unwrap();
        assert_eq!(results.len(), 1);

        let query = ListQuery::default().with_pattern("gts.other.*");
        let results = repo.list(&query).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_list_with_segment_scope_primary() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        repo.register(&entity, false).unwrap();
        repo.switch_to_ready().unwrap();

        let query = ListQuery::default()
            .with_vendor("acme")
            .with_segment_scope(SegmentMatchScope::Primary);
        let results = repo.list(&query).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_register_with_description() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "description": "A user created event"
        });

        let result = repo.register(&entity, false).unwrap();
        assert_eq!(result.description, Some("A user created event".to_owned()));
    }

    #[test]
    fn test_register_instance() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "id": "gts.acme.core.events.user_created.v1~acme.core.events.instance.v1",
            "data": "value"
        });

        let result = repo.register(&entity, false).unwrap();
        assert!(result.is_instance());
    }

    #[test]
    fn test_extract_gts_id_with_gtsid_field() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "gtsId": "gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        let result = repo.register(&entity, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_gts_id_with_id_field() {
        let repo = InMemoryGtsRepository::new(default_config());

        let entity = json!({
            "id": "gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        let result = repo.register(&entity, false);
        assert!(result.is_ok());
    }
}
