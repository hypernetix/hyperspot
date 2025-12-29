//! Test case: schema_for! on GTS-wrapped struct should trigger DE0902

use gts_macros::struct_to_gts_schema;
use schemars;

/// A GTS-wrapped struct (has struct_to_gts_schema attribute)
#[derive(Debug, Clone)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.plugin.v1~",
    description = "Test plugin specification",
    properties = "id,vendor"
)]
pub struct MyGtsPluginSpecV1 {
    pub id: gts::GtsInstanceId,
    pub vendor: String,
}

fn main() {
    // Should trigger DE0902 - schema_for on GTS struct
    let _schema = schemars::schema_for!(MyGtsPluginSpecV1);
}
