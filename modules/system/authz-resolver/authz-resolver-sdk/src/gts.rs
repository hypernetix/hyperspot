//! GTS schema definitions for `AuthZ` resolver plugins.

use gts_macros::struct_to_gts_schema;
use modkit::gts::BaseModkitPluginV1;

/// GTS type definition for `AuthZ` resolver plugin instances.
///
/// # Instance ID Format
///
/// ```text
/// gts.x.core.modkit.plugin.v1~<vendor>.<package>.authz_resolver.plugin.v1~
/// ```
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseModkitPluginV1,
    schema_id = "gts.x.core.modkit.plugin.v1~x.core.authz_resolver.plugin.v1~",
    description = "AuthZ Resolver plugin specification",
    properties = ""
)]
pub struct AuthZResolverPluginSpecV1;
