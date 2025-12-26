// Test file for gts-macros runtime API calls - should not trigger DE0901

use gts::GtsInstanceId;
use gts_macros::struct_to_gts_schema;

#[derive(Debug)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.entities.product.v1~",
    description = "Product entity",
    properties = "id"
)]
pub struct ProductV1<P: gts::GtsSchema> {
    pub id: GtsInstanceId,
    pub properties: P,
}

fn main() {
    // gts_make_instance_id takes a segment (NOT a full gts.* string)
    // Should not trigger DE0901
    let _id = ProductV1::<()>::gts_make_instance_id("vendor.package.sku.abc.v1");
}
