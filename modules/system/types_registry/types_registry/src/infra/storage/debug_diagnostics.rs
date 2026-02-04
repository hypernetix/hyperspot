//! Debug diagnostics for GTS entity registration failures.
//!
//! This module provides helper functions to emit debug-level logs when
//! GTS entity registration or validation fails. The logs include:
//! - The complete entity content being registered
//! - The schema chain for instance validation failures
//! - Cycle detection for circular schema references

use std::collections::HashSet;

use gts::GtsOps;
use serde_json::Value;
use tracing::{debug, warn};

/// Logs the entity content when registration fails.
///
/// Emits a debug log with the complete entity JSON (pretty-printed)
/// and the GTS ID if available.
pub fn log_registration_failure(gts_id: Option<&str>, entity: &Value, error: &str) {
    let entity_json = serde_json::to_string_pretty(entity).unwrap_or_else(|_| entity.to_string());

    if let Some(id) = gts_id {
        debug!(
            gts_id = %id,
            error = %error,
            "GTS entity registration failed.\nEntity content:\n{}",
            entity_json
        );
    } else {
        debug!(
            error = %error,
            "GTS entity registration failed (no GTS ID found).\nEntity content:\n{}",
            entity_json
        );
    }
}

/// Logs the schema validation failure with the schema content.
///
/// Emits a debug log with the complete schema JSON (pretty-printed).
pub fn log_schema_validation_failure(gts_id: &str, schema: &Value, error: &str) {
    let schema_json = serde_json::to_string_pretty(schema).unwrap_or_else(|_| schema.to_string());

    debug!(
        gts_id = %gts_id,
        error = %error,
        "GTS schema validation failed.\nSchema content:\n{}",
        schema_json
    );
}

/// Logs the instance validation failure with the instance and schema chain.
///
/// Emits debug logs with:
/// - The complete instance JSON
/// - Each schema in the inheritance chain with depth labels
pub fn log_instance_validation_failure(
    gts_id: &str,
    instance: &Value,
    error: &str,
    ops: &mut GtsOps,
) {
    let instance_json =
        serde_json::to_string_pretty(instance).unwrap_or_else(|_| instance.to_string());

    debug!(
        gts_id = %gts_id,
        error = %error,
        "GTS instance validation failed.\nInstance content:\n{}",
        instance_json
    );

    // Try to find the schema chain for this instance
    if let Some(schema_id) = extract_schema_id(gts_id) {
        log_schema_chain(ops, &schema_id);
    }
}

/// Extracts the schema ID from an instance GTS ID.
///
/// For instance IDs like `gts.vendor.pkg.ns.type.v1~vendor.app.instance.v1`,
/// the schema ID is the type portion before the first non-tilde segment.
/// Returns `None` if the ID is a schema ID (ends with `~`) or has no `~`.
fn extract_schema_id(instance_gts_id: &str) -> Option<String> {
    // Instance ID format: type_id~instance_suffix
    // The schema ID is everything up to and including the first ~
    // Only return if there's content after the ~ (i.e., it's an instance, not a schema)
    if let Some(tilde_pos) = instance_gts_id.find('~')
        && tilde_pos + 1 < instance_gts_id.len()
    {
        return Some(instance_gts_id[..=tilde_pos].to_owned());
    }
    None
}

/// Logs the schema inheritance chain for debugging.
///
/// Walks the schema chain via `$ref` fields and logs each schema
/// with its depth/role in the chain.
pub fn log_schema_chain(ops: &mut GtsOps, schema_id: &str) {
    let mut visited: HashSet<String> = HashSet::new();
    log_schema_chain_recursive(ops, schema_id, &mut visited, 0);
}

fn log_schema_chain_recursive(
    ops: &mut GtsOps,
    schema_id: &str,
    visited: &mut HashSet<String>,
    depth: usize,
) {
    // Cycle detection
    if check_and_mark_visited(visited, schema_id, depth) {
        return;
    }

    // Try to get the schema from the store
    let Some(schema_content) = try_get_schema(ops, schema_id, depth) else {
        return;
    };

    // Log the schema content
    log_schema_content(schema_id, &schema_content, depth);

    // Walk $ref and allOf references
    for ref_id in collect_schema_refs(&schema_content) {
        log_schema_chain_recursive(ops, &ref_id, visited, depth + 1);
    }
}

/// Check if schema was already visited (cycle detection) and mark as visited
fn check_and_mark_visited(visited: &mut HashSet<String>, schema_id: &str, depth: usize) -> bool {
    if visited.contains(schema_id) {
        warn!(
            schema_id = %schema_id,
            depth = depth,
            "Cycle detected in schema chain at ID: {}",
            schema_id
        );
        return true;
    }
    visited.insert(schema_id.to_owned());
    false
}

/// Try to retrieve schema content from the store
fn try_get_schema(ops: &mut GtsOps, schema_id: &str, depth: usize) -> Option<Value> {
    if let Some(entity) = ops.store.get(schema_id) {
        Some(entity.content.clone())
    } else {
        debug!(
            schema_id = %schema_id,
            depth = depth,
            "Schema not found in store"
        );
        None
    }
}

/// Log the schema content with depth and role information
fn log_schema_content(schema_id: &str, schema_content: &Value, depth: usize) {
    let schema_json =
        serde_json::to_string_pretty(schema_content).unwrap_or_else(|_| schema_content.to_string());

    let role = if depth == 0 {
        "Instance Schema"
    } else {
        "Ref Schema"
    };

    debug!(
        schema_id = %schema_id,
        "Depth {} ({}):\n{}",
        depth,
        role,
        schema_json
    );
}

/// Collects all schema references from a JSON Schema.
///
/// Looks for:
/// - `$ref` fields pointing to GTS IDs
/// - `allOf` arrays containing `$ref` entries
fn collect_schema_refs(schema: &Value) -> Vec<String> {
    let mut refs = Vec::new();

    if let Some(obj) = schema.as_object() {
        // Direct $ref
        if let Some(ref_val) = obj.get("$ref").and_then(|v| v.as_str())
            && let Some(gts_ref) = normalize_gts_ref(ref_val)
        {
            refs.push(gts_ref);
        }

        // allOf array
        if let Some(all_of) = obj.get("allOf").and_then(|v| v.as_array()) {
            for item in all_of {
                if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str())
                    && let Some(gts_ref) = normalize_gts_ref(ref_val)
                {
                    refs.push(gts_ref);
                }
            }
        }

        // x-gts-ref (GTS-specific reference)
        if let Some(ref_val) = obj.get("x-gts-ref").and_then(|v| v.as_str())
            && let Some(gts_ref) = normalize_gts_ref(ref_val)
        {
            refs.push(gts_ref);
        }
    }

    refs
}

/// Normalizes a reference to a GTS ID.
///
/// Handles both:
/// - Direct GTS IDs: `gts.vendor.pkg.ns.type.v1~`
/// - URI format: `gts://gts.vendor.pkg.ns.type.v1~`
fn normalize_gts_ref(ref_val: &str) -> Option<String> {
    let cleaned = ref_val.strip_prefix("gts://").unwrap_or(ref_val);

    // Only return if it looks like a GTS ID
    if cleaned.starts_with("gts.") {
        Some(cleaned.to_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_schema_id() {
        assert_eq!(
            extract_schema_id("gts.vendor.pkg.ns.type.v1~vendor.app.instance.v1"),
            Some("gts.vendor.pkg.ns.type.v1~".to_owned())
        );
        // Schema IDs (ending with ~) don't have an "instance" portion
        assert_eq!(extract_schema_id("gts.vendor.pkg.ns.type.v1~"), None);
        assert_eq!(extract_schema_id("no-tilde"), None);
    }

    #[test]
    fn test_normalize_gts_ref() {
        assert_eq!(
            normalize_gts_ref("gts://gts.vendor.pkg.ns.type.v1~"),
            Some("gts.vendor.pkg.ns.type.v1~".to_owned())
        );
        assert_eq!(
            normalize_gts_ref("gts.vendor.pkg.ns.type.v1~"),
            Some("gts.vendor.pkg.ns.type.v1~".to_owned())
        );
        assert_eq!(normalize_gts_ref("#/definitions/Something"), None);
        assert_eq!(normalize_gts_ref("http://example.com/schema"), None);
    }

    #[test]
    fn test_collect_schema_refs() {
        let schema = json!({
            "$ref": "gts://gts.vendor.pkg.ns.base.v1~",
            "allOf": [
                { "$ref": "gts.vendor.pkg.ns.mixin.v1~" }
            ],
            "x-gts-ref": "gts.vendor.pkg.ns.other.v1~"
        });

        let refs = collect_schema_refs(&schema);
        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&"gts.vendor.pkg.ns.base.v1~".to_owned()));
        assert!(refs.contains(&"gts.vendor.pkg.ns.mixin.v1~".to_owned()));
        assert!(refs.contains(&"gts.vendor.pkg.ns.other.v1~".to_owned()));
    }

    #[test]
    fn test_collect_schema_refs_empty() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        });

        let refs = collect_schema_refs(&schema);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_cycle_detection_in_visited_set() {
        let mut visited: HashSet<String> = HashSet::new();
        let schema_id = "gts.vendor.pkg.ns.type.v1~";

        // First visit should succeed
        assert!(!visited.contains(schema_id));
        visited.insert(schema_id.to_owned());

        // Second visit should be detected as cycle
        assert!(visited.contains(schema_id));
    }

    #[test]
    fn test_log_registration_failure_with_gts_id() {
        // This test verifies the function doesn't panic
        let entity = json!({
            "$id": "gts://gts.acme.core.events.test.v1~",
            "type": "object"
        });
        log_registration_failure(Some("gts.acme.core.events.test.v1~"), &entity, "Test error");
    }

    #[test]
    fn test_log_registration_failure_without_gts_id() {
        // This test verifies the function doesn't panic
        let entity = json!({
            "type": "object"
        });
        log_registration_failure(None, &entity, "No GTS ID found");
    }

    #[test]
    fn test_log_schema_validation_failure() {
        // This test verifies the function doesn't panic
        let schema = json!({
            "$id": "gts://gts.acme.core.events.test.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "invalid_type"
        });
        log_schema_validation_failure("gts.acme.core.events.test.v1~", &schema, "Invalid type");
    }

    #[test]
    fn test_extract_schema_id_with_chained_instance() {
        // Instance with multiple segments
        assert_eq!(
            extract_schema_id("gts.a.b.c.d.v1~vendor.app.x.y.v1"),
            Some("gts.a.b.c.d.v1~".to_owned())
        );
    }

    #[test]
    fn test_collect_schema_refs_nested_allof() {
        let schema = json!({
            "allOf": [
                { "$ref": "gts.vendor.pkg.ns.base1.v1~" },
                { "$ref": "gts.vendor.pkg.ns.base2.v1~" },
                { "type": "object" }
            ]
        });

        let refs = collect_schema_refs(&schema);
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&"gts.vendor.pkg.ns.base1.v1~".to_owned()));
        assert!(refs.contains(&"gts.vendor.pkg.ns.base2.v1~".to_owned()));
    }
}
