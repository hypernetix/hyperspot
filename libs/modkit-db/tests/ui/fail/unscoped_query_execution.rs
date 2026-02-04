//! Compile-fail test: Unscoped queries cannot be executed.
//!
//! This test verifies that `SecureSelect` in `Unscoped` state cannot call `.all()`,
//! `.one()`, or other execution methods. Only `Scoped` state allows execution.
//!
//! Security: The typestate pattern ensures compile-time enforcement of scope
//! requirements. Without calling `.scope_with()`, queries cannot be executed,
//! preventing accidental unscoped data access.

use modkit_db::secure::{SecureEntityExt, DBRunner};
use sea_orm::entity::prelude::*;

// Minimal entity definition for the test
mod test_entity {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "test_table")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

use test_entity::Entity;

async fn attempt_unscoped_query<C: DBRunner>(conn: &C) {
    // ERROR: .all() is not available on Unscoped state
    // Must call .scope_with(&scope) first to transition to Scoped state
    let _result = Entity::find().secure().all(conn).await;
}

fn main() {}
