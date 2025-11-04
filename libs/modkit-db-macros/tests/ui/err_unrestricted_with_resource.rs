// Using unrestricted with resource_col should produce a compile error.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(unrestricted, resource_col = "id")]
struct Model;

fn main() {}

