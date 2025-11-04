// Using unrestricted with owner_col should produce a compile error.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(unrestricted, owner_col = "owner_id")]
struct Model;

fn main() {}

