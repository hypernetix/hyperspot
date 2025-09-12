use sea_orm_migration::prelude::*;

mod initial_001;
mod initial_settings_001;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(initial_001::Migration),
            Box::new(initial_settings_001::Migration),
        ]
    }
}
