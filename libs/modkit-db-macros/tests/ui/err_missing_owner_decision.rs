// Missing explicit owner decision should produce a compile error.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    no_type
)]
struct Model;

fn main() {}

