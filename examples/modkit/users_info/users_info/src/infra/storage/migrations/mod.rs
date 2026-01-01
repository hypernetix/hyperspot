use sea_orm_migration::prelude::*;

mod add_relationships_003;
mod add_tenant_support_002;
mod add_tenant_to_all_tables_004;
mod initial_001;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(initial_001::Migration),
            Box::new(add_tenant_support_002::Migration),
            Box::new(add_relationships_003::Migration),
            Box::new(add_tenant_to_all_tables_004::Migration),
        ]
    }
}
