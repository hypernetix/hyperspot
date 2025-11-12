// Using unrestricted with type_col should produce a compile error.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(unrestricted, type_col = "type_id")]
struct Model;

fn main() {}

