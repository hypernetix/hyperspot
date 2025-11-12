// All columns explicitly specified - macro should expand without errors.
// Note: This test only validates macro expansion, not the full trait implementation.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    owner_col = "owner_id",
    type_col = "type_id"
)]
struct Model {
    tenant_id: String,
    id: String,
    owner_id: String,
    type_id: String,
}

fn main() {}

