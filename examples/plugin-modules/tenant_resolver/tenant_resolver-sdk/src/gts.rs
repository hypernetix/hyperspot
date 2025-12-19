//! GTS schema definitions for `tenant_resolver` example.
//!
//! This module contains GTS type definitions that are registered with
//! the types-registry and used for schema validation.

use gts_macros::struct_to_gts_schema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// GTS type definition for tenant resolver plugin instances.
///
/// This type represents the schema for plugin instances that implement
/// the tenant resolver API. Each plugin registers an instance of this type.
///
/// GTS ID format: `gts.x.core.plugins.thr_plugin.v1~`
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.plugins.thr_plugin.v1~",
    description = "Tenant resolver plugin specification",
    properties = "id,vendor,priority"
)]
pub struct ThrPluginSpec {
    /// Well-known instance identifier (full GTS instance ID, no trailing `~`).
    ///
    /// Example: `gts.x.core.plugins.thr_plugin.v1~contoso.plugins._.thr_plugin.v1`
    pub id: String,
    /// Vendor name for the plugin.
    pub vendor: String,
    /// Priority for plugin selection (lower = higher priority).
    pub priority: i16,
}

/// GTS type definition for tenants.
///
/// GTS ID format: `gts.x.core.tenants.tenant.v1~`
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.tenants.tenant.v1~",
    description = "Tenant entity specification",
    properties = "id,name,description"
)]
pub struct TenantSpec {
    /// Well-known instance identifier (full GTS instance ID, no trailing `~`).
    pub id: String,
    /// Tenant display name.
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
