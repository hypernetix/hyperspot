// Test file with invalid GTS schema_id strings - should trigger DE0901

use gts::GtsInstanceId;
use gts_macros::struct_to_gts_schema;

#[derive(Debug)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    // Should trigger DE0901 - invalid GTS schema_id string
    schema_id = "gts.x.core.events.type.v1",
    description = "Missing trailing tilde",
    properties = "id"
)]
pub struct MissingTildeV1<P: gts::GtsSchema> {
    pub id: GtsInstanceId,
    pub properties: P,
}

#[derive(Debug)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    // Should trigger DE0901 - invalid GTS schema_id string
    schema_id = "gts.x.core.events-type.v1~",
    description = "Invalid hyphen in component",
    properties = "id"
)]
pub struct InvalidCharV1<P: gts::GtsSchema> {
    pub id: GtsInstanceId,
    pub properties: P,
}

fn main() {}
