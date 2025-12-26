use gts::GtsInstanceId;
use gts_macros::struct_to_gts_schema;

#[derive(Debug)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.modkit.plugin.v1~",
    description = "Base modkit plugin schema",
    properties = "id,vendor,priority,properties"
)]
pub struct BaseModkitPluginV1<P: gts::GtsSchema> {
    pub id: GtsInstanceId, // Full GTS instance ID
    pub vendor: String,    // Vendor name for selection
    pub priority: i16,     // Lower = higher priority
    pub properties: P,
}
