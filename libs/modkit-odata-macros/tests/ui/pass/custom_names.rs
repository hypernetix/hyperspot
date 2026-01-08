use modkit_odata_macros::ODataSchema;
use modkit_odata::schema::Schema;

#[derive(ODataSchema)]
struct Product {
    #[odata(name = "product_id")]
    id: uuid::Uuid,
    #[odata(name = "product_name")]
    name: String,
}

fn main() {
    assert_eq!(ProductSchema::field_name(ProductField::Id), "product_id");
    assert_eq!(ProductSchema::field_name(ProductField::Name), "product_name");
}
