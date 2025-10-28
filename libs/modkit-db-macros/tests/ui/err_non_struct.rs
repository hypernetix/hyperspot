// Derive macro applied to a non-struct should abort.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
enum NotAStruct {
    A,
    B,
}

