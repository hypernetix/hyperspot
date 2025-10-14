// Unknown attribute key should abort with a clear message.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(does_not_exist = "oops")]
struct Model;

