//! GTS schema definitions for `AuthN` resolver plugins.
//!
//! This module defines the GTS type for `AuthN` resolver plugin instances.
//! Plugins register instances of this type with the types-registry to be
//! discovered by the gateway.

use gts_macros::struct_to_gts_schema;
use modkit::gts::BaseModkitPluginV1;

/// GTS type definition for `AuthN` resolver plugin instances.
///
/// Each plugin registers an instance of this type with its vendor-specific
/// instance ID. The gateway discovers plugins by querying types-registry
/// for instances matching this schema.
///
/// # Instance ID Format
///
/// ```text
/// gts.x.core.modkit.plugin.v1~<vendor>.<package>.authn_resolver.plugin.v1~
/// ```
///
/// # Example
///
/// ```ignore
/// // Plugin generates its instance ID
/// let instance_id = AuthNResolverPluginSpecV1::gts_make_instance_id(
///     "hyperspot.builtin.static_authn_resolver.plugin.v1"
/// );
///
/// // Plugin creates instance data
/// let instance = BaseModkitPluginV1::<AuthNResolverPluginSpecV1> {
///     id: instance_id.clone(),
///     vendor: "hyperspot".to_owned(),
///     priority: 100,
///     properties: AuthNResolverPluginSpecV1,
/// };
///
/// // Register with types-registry
/// registry.register(vec![serde_json::to_value(&instance)?]).await?;
/// ```
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseModkitPluginV1,
    schema_id = "gts.x.core.modkit.plugin.v1~x.core.authn_resolver.plugin.v1~",
    description = "AuthN Resolver plugin specification",
    properties = ""
)]
pub struct AuthNResolverPluginSpecV1;
