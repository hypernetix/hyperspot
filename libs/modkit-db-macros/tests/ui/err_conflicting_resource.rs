// Both resource_col and no_resource specified should produce a compile error.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    no_resource,
    no_owner,
    no_type
)]
struct Model;

fn main() {}

