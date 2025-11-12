// Unrestricted flag with no other attributes - macro should expand without errors.
// Note: This test only validates macro expansion, not the full trait implementation.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(unrestricted)]
struct Model {
    id: String,
}

fn main() {}

