//! GTS schema definitions for `tenant_resolver` example.
//!
//! This module contains GTS type definitions that are registered with
//! the types-registry and used for schema validation.

use gts::GtsInstanceId;
use gts_macros::struct_to_gts_schema;
use modkit::gts::BaseModkitPluginV1;

/// GTS type definition for tenant resolver plugin instances.
///
/// This type represents the schema for plugin instances that implement
/// the tenant resolver API. Each plugin registers an instance of this type.
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseModkitPluginV1,
    schema_id = "gts.x.core.modkit.plugin.v1~x.core.tenant_resolver.plugin.v1~",
    description = "Tenant resolver plugin specification",
    properties = ""
)]
pub struct TenantResolverPluginSpecV1;

/// GTS type definition for tenants.
///
/// GTS ID format: `gts.x.core.tenants.tenant.v1~`
#[derive(Debug, Clone)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.tenants.tenant.v1~",
    description = "Tenant entity specification",
    properties = "id,name,description,properties"
)]
pub struct TenantSpecV1<T: gts::GtsSchema> {
    /// Well-known instance identifier (full GTS instance ID, no trailing `~`).
    pub id: GtsInstanceId,
    /// Tenant display name.
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub properties: T,
}
