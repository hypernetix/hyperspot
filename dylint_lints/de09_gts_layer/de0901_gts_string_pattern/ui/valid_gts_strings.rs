// Test file with valid GTS schema_id strings - should not trigger DE0901

use gts::GtsInstanceId;
use gts_macros::struct_to_gts_schema;

#[derive(Debug)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.events.type.v1~",
    description = "Base event type definition",
    properties = "id"
)]
pub struct BaseEventTypeV1<P: gts::GtsSchema> {
    pub id: GtsInstanceId,
    pub properties: P,
}

#[derive(Debug)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseEventTypeV1,
    schema_id = "gts.x.core.events.type.v1~x.core.audit.event.v1~",
    description = "Audit event",
    properties = "user_id"
)]
pub struct AuditEventV1 {
    pub user_id: String,
}

fn main() {
    // Should NOT trigger DE0901
    let _s1 = "gts.x.core.events.type.v1~";
    // Should NOT trigger DE0901
    let _s2 = "gts.x.core.events.type.v1~x.core.audit.event.v1~";
}
