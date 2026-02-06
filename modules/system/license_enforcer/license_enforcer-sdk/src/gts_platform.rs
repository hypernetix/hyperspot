//! GTS schema definitions for license enforcer platform plugins.
//!
//! This module defines the GTS type for platform plugin instances.
//! Plugins register instances of this type with the types-registry to be
//! discovered by the gateway.

use gts_macros::struct_to_gts_schema;
use modkit::gts::BaseModkitPluginV1;

/// GTS type definition for license enforcer platform plugin instances.
///
/// Each platform plugin registers an instance of this type with its vendor-specific
/// instance ID. The gateway discovers plugins by querying types-registry
/// for instances matching this schema.
///
/// # Schema ID
///
/// ```text
/// gts.x.core.modkit.plugin.v1~x.core.license_resolver.plugin.v1~
/// ```
///
/// # Instance ID Format
///
/// ```text
/// gts.x.core.modkit.plugin.v1~x.core.license_resolver.plugin.v1~<vendor>.<package>.integration_plugin.v1
/// ```
///
/// # Example
///
/// ```
/// use license_enforcer_sdk::LicensePlatformPluginSpecV1;
/// use modkit::gts::BaseModkitPluginV1;
///
/// // Plugin generates its instance ID
/// let instance_id = LicensePlatformPluginSpecV1::gts_make_instance_id(
///     "hyperspot.static_licenses.integration_plugin.v1"
/// );
///
/// // Plugin creates instance data
/// let instance = BaseModkitPluginV1::<LicensePlatformPluginSpecV1> {
///     id: instance_id.clone(),
///     vendor: "hyperspot".to_string(),
///     priority: 100,
///     properties: LicensePlatformPluginSpecV1,
/// };
///
/// // Serialize for registration
/// let _json = serde_json::to_value(&instance).unwrap();
/// // Then register with types-registry: registry.register(vec![json]).await
/// ```
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseModkitPluginV1,
    schema_id = "gts.x.core.modkit.plugin.v1~x.core.license_resolver.plugin.v1~",
    description = "License Enforcer platform integration plugin specification",
    properties = ""
)]
pub struct LicensePlatformPluginSpecV1;
