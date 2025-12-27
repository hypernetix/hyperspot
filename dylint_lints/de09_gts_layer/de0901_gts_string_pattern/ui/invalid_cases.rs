// Test file for invalid schema_id in struct_to_gts_schema attributes

use gts::GtsInstanceId;
use gts_macros::struct_to_gts_schema;

#[derive(Debug)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    // Should NOT trigger - valid GTS schema_id string
    schema_id = "gts.x.test.entities.product.v1~",
    description = "Product entity",
    properties = "id"
)]
pub struct ProductV1<P: gts::GtsSchema> {
    pub id: GtsInstanceId,
    pub properties: P,
}

#[derive(Debug)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    // Should trigger DE0901 - invalid GTS schema_id (missing trailing tilde)
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
    // Should trigger DE0901 - invalid GTS schema_id (invalid hyphen)
    schema_id = "gts.x.core.events-type.v1~",
    description = "Invalid hyphen in component",
    properties = "id"
)]
pub struct InvalidCharV1<P: gts::GtsSchema> {
    pub id: GtsInstanceId,
    pub properties: P,
}

#[derive(Debug)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    // Should trigger DE0901 - invalid GTS schema_id (wildcard not allowed)
    schema_id = "gts.x.*.events.type.v1~",
    description = "Wildcards not allowed in schema_id",
    properties = "id"
)]
pub struct WildcardSchemaV1<P: gts::GtsSchema> {
    pub id: GtsInstanceId,
    pub properties: P,
}

fn main() {
    // Error 1: Incomplete chained segments (missing type component)
    // Should trigger DE0901 - invalid GTS string
    let _id1 = ProductV1::<()>::gts_make_instance_id("vendor.package.sku.abc.v1~a.b.c");

    // Error 2: Incomplete segment (missing type component)
    // Should trigger DE0901 - invalid GTS format
    let _id2 = ProductV1::<()>::gts_make_instance_id("vendor.package.sku.v1");

    // Error 3: Type schema (ends with ~) - gts_make_instance_id must not accept schemas
    // Should trigger DE0901 - invalid GTS entity type (schema instead of instance)
    let _id3 = ProductV1::<()>::gts_make_instance_id("vendor.package.sku.abc.v1~");

    // Error 4: Wildcard - gts_make_instance_id must not accept wildcards
    // Should trigger DE0901 - invalid GTS format
    let _id4 = ProductV1::<()>::gts_make_instance_id("vendor.package.*.abc.v1");

    // Error 5: Multiple segments (contains ~) - gts_make_instance_id must accept only ONE instance segment
    // Should trigger DE0901 - invalid GTS string
    let _id1 = ProductV1::<()>::gts_make_instance_id("vendor.package.sku.abc.v1~a.b.c.d.v1");

    // Error 6: invalid GTS segment
    // Should trigger DE0901 - invalid GTS segment
    let _s = "gts.x.core.lic.feat.v1~x.core.global.base";

    // Error 7: Invalid GTS identifier (no trailing type segment)
    // Should trigger DE0901 - invalid GTS indentifier
    let _s = "gts.x.core.events.type.v1";

    // Error 8: GTS wildcard is not allowed in regular strings
    // Should trigger DE0901 - invalid GTS
    let _s = "gts.x.core.events.type.*";

    // Valid case for comparison
    // Should NOT trigger - valid GTS instance segment
    let _id_valid = ProductV1::<()>::gts_make_instance_id("vendor.package.sku.abc.v1");
}
