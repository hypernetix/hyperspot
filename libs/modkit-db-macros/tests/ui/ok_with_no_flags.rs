// Using no_* flags explicitly - macro should expand without errors.
// Note: This test only validates macro expansion, not the full trait implementation.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    no_owner,
    no_type
)]
struct Model {
    tenant_id: String,
    id: String,
}

fn main() {}

