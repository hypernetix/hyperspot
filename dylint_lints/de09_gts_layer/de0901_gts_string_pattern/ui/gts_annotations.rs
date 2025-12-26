// Test file for gts-macros annotations - should not trigger DE0901

use gts::GtsInstanceId;
use gts_macros::struct_to_gts_schema;

#[derive(Debug)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.events.topic.v1~",
    description = "Event Topic definition",
    properties = "id,name"
)]
pub struct EventTopicV1<T: gts::GtsSchema> {
    pub id: GtsInstanceId,
    pub name: String,
    pub properties: T,
}

fn main() {
    // Should not trigger DE0901
    let _id = EventTopicV1::<()>::gts_make_instance_id("x.commerce.orders.orders.v1.0");
}
