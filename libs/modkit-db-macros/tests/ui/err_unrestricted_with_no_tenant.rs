// Using unrestricted with no_tenant should produce a compile error.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(unrestricted, no_tenant)]
struct Model;

fn main() {}

