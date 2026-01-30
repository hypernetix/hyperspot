//! GTS schema definitions for license enforcer cache plugins.
//!
//! This module defines the GTS type for cache plugin instances.
//! Plugins register instances of this type with the types-registry to be
//! discovered by the gateway.

use gts_macros::struct_to_gts_schema;
use modkit::gts::BaseModkitPluginV1;

/// GTS type definition for license enforcer cache plugin instances.
///
/// Each cache plugin registers an instance of this type with its vendor-specific
/// instance ID. The gateway discovers plugins by querying types-registry
/// for instances matching this schema.
///
/// # Schema ID
///
/// ```text
/// gts.x.core.modkit.plugin.v1~x.core.license_enforcer.cache.plugin.v1~
/// ```
///
/// # Instance ID Format
///
/// ```text
/// gts.x.core.modkit.plugin.v1~x.core.license_enforcer.cache.plugin.v1~<vendor>.<package>.cache.plugin.v1
/// ```
///
/// # Example
///
/// ```ignore
/// // Plugin generates its instance ID
/// let instance_id = LicenseCachePluginSpecV1::gts_make_instance_id(
///     "hyperspot.builtin.nocache.plugin.v1"
/// );
///
/// // Plugin creates instance data
/// let instance = BaseModkitPluginV1::<LicenseCachePluginSpecV1> {
///     id: instance_id.clone(),
///     vendor: "hyperspot".to_string(),
///     priority: 100,
///     properties: LicenseCachePluginSpecV1,
/// };
///
/// // Register with types-registry
/// registry.register(vec![serde_json::to_value(&instance)?]).await?;
/// ```
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseModkitPluginV1,
    schema_id = "gts.x.core.modkit.plugin.v1~x.core.license_enforcer.cache.plugin.v1~",
    description = "License Enforcer cache plugin specification",
    properties = ""
)]
pub struct LicenseCachePluginSpecV1;
