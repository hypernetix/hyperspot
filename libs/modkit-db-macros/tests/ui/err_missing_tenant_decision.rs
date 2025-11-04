// Missing explicit tenant decision should produce a compile error.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    resource_col = "id",
    no_owner,
    no_type
)]
struct Model;

fn main() {}

