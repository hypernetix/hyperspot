use sea_orm_migration::prelude::*;

mod m20260111_000001_initial;
mod m20260111_000002_add_tenant_support;
mod m20260111_000003_add_relationships;
mod m20260111_000004_add_tenant_to_all_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260111_000001_initial::Migration),
            Box::new(m20260111_000002_add_tenant_support::Migration),
            Box::new(m20260111_000003_add_relationships::Migration),
            Box::new(m20260111_000004_add_tenant_to_all_tables::Migration),
        ]
    }
}
