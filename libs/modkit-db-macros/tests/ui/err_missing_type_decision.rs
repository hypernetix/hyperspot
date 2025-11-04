// Missing explicit type decision should produce a compile error.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    no_owner
)]
struct Model;

fn main() {}

