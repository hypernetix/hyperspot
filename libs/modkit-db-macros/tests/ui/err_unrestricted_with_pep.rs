// pep_prop cannot be used with unrestricted.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(unrestricted, pep_prop(custom = "custom_col"))]
struct Model;
