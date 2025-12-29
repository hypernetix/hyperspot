use crate::gts::BaseModkitPluginV1;
use tracing::info;

/// Returns the core GTS schemas provided by the modkit framework.
///
/// These are base types that other modules' plugin systems depend on
///
/// This function is called by `types_registry` during initialization to register
/// these core types before any dependent modules can register their derived schemas.
///
/// NOTE: This is temporary logic until <https://github.com/hypernetix/hyperspot/issues/156> is resolved
///
/// # Errors
///
/// Returns an error if the schema JSON cannot be parsed.
pub fn get_core_gts_schemas() -> anyhow::Result<Vec<serde_json::Value>> {
    info!("Generating core GTS schemas");

    let mut schemas = Vec::new();

    // BaseModkitPluginV1 schema (gts.x.core.modkit.plugin.v1~)
    // This is the base type for all plugin schemas in the modkit framework.
    let schema_str = BaseModkitPluginV1::<()>::gts_schema_with_refs_as_string();
    let schema_json: serde_json::Value = serde_json::from_str(&schema_str)?;
    schemas.push(schema_json);

    info!("Core GTS schemas generated: gts.x.core.modkit.plugin.v1~");
    Ok(schemas)
}
