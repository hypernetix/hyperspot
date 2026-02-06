//! Configuration for the Types Registry module.

use serde::Deserialize;

/// Configuration for the Types Registry module.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct TypesRegistryConfig {
    /// Fields to check for GTS entity ID (in order of priority).
    /// Default: `["$id", "gtsId", "id"]`
    pub entity_id_fields: Vec<String>,

    /// Fields to check for schema ID reference (in order of priority).
    /// Default: `["$schema", "gtsTid", "type"]`
    pub schema_id_fields: Vec<String>,
}

impl Default for TypesRegistryConfig {
    fn default() -> Self {
        Self {
            entity_id_fields: vec!["$id".to_owned(), "gtsId".to_owned(), "id".to_owned()],
            schema_id_fields: vec!["$schema".to_owned(), "gtsTid".to_owned(), "type".to_owned()],
        }
    }
}

impl TypesRegistryConfig {
    /// Converts this config to a `gts::GtsConfig`.
    #[must_use]
    pub fn to_gts_config(&self) -> gts::GtsConfig {
        gts::GtsConfig {
            entity_id_fields: self.entity_id_fields.clone(),
            schema_id_fields: self.schema_id_fields.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = TypesRegistryConfig::default();
        assert_eq!(cfg.entity_id_fields, vec!["$id", "gtsId", "id"]);
        assert_eq!(cfg.schema_id_fields, vec!["$schema", "gtsTid", "type"]);
    }

    #[test]
    fn test_to_gts_config() {
        let cfg = TypesRegistryConfig::default();
        let gts_cfg = cfg.to_gts_config();
        assert_eq!(gts_cfg.entity_id_fields, cfg.entity_id_fields);
        assert_eq!(gts_cfg.schema_id_fields, cfg.schema_id_fields);
    }
}
