// Missing explicit resource decision should produce a compile error.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    no_owner,
    no_type
)]
struct Model;

fn main() {}

