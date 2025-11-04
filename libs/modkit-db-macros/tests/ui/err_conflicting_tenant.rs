// Both tenant_col and no_tenant specified should produce a compile error.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    no_tenant,
    resource_col = "id",
    no_owner,
    no_type
)]
struct Model;

fn main() {}

