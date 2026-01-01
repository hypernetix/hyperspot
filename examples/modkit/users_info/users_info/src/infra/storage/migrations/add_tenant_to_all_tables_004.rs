use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let conn = manager.get_connection();

        let sql = match backend {
            sea_orm::DatabaseBackend::Postgres => {
                r#"
-- Add tenant_id to cities table
ALTER TABLE cities ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_cities_tenant ON cities(tenant_id);

-- Add tenant_id to languages table
ALTER TABLE languages ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_languages_tenant ON languages(tenant_id);

-- Add tenant_id to addresses table
ALTER TABLE addresses ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_addresses_tenant ON addresses(tenant_id);

-- Add tenant_id to users_languages table
ALTER TABLE users_languages ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_users_languages_tenant ON users_languages(tenant_id);
                "#
            }
            sea_orm::DatabaseBackend::MySql => {
                r#"
-- Add tenant_id to cities table
ALTER TABLE cities ADD COLUMN tenant_id VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX idx_cities_tenant ON cities(tenant_id);

-- Add tenant_id to languages table
ALTER TABLE languages ADD COLUMN tenant_id VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX idx_languages_tenant ON languages(tenant_id);

-- Add tenant_id to addresses table
ALTER TABLE addresses ADD COLUMN tenant_id VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX idx_addresses_tenant ON addresses(tenant_id);

-- Add tenant_id to users_languages table
ALTER TABLE users_languages ADD COLUMN tenant_id VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX idx_users_languages_tenant ON users_languages(tenant_id);
                "#
            }
            sea_orm::DatabaseBackend::Sqlite => {
                r#"
-- SQLite doesn't support ALTER TABLE ADD COLUMN IF NOT EXISTS, so we need to check first
-- For SQLite, we'll use a different approach with CREATE TABLE IF NOT EXISTS and data migration

-- Add tenant_id to cities table
ALTER TABLE cities ADD COLUMN tenant_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_cities_tenant ON cities(tenant_id);

-- Add tenant_id to languages table
ALTER TABLE languages ADD COLUMN tenant_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_languages_tenant ON languages(tenant_id);

-- Add tenant_id to addresses table
ALTER TABLE addresses ADD COLUMN tenant_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_addresses_tenant ON addresses(tenant_id);

-- Add tenant_id to users_languages table
ALTER TABLE users_languages ADD COLUMN tenant_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_users_languages_tenant ON users_languages(tenant_id);
                "#
            }
        };

        conn.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let conn = manager.get_connection();

        let sql = match backend {
            sea_orm::DatabaseBackend::Postgres => {
                r#"
ALTER TABLE cities DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE languages DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE addresses DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE users_languages DROP COLUMN IF EXISTS tenant_id;
                "#
            }
            sea_orm::DatabaseBackend::MySql => {
                r#"
ALTER TABLE cities DROP COLUMN tenant_id;
ALTER TABLE languages DROP COLUMN tenant_id;
ALTER TABLE addresses DROP COLUMN tenant_id;
ALTER TABLE users_languages DROP COLUMN tenant_id;
                "#
            }
            sea_orm::DatabaseBackend::Sqlite => {
                r#"
-- SQLite doesn't support DROP COLUMN easily, would need table recreation
-- For now, we'll leave the columns (SQLite limitation)
                "#
            }
        };

        conn.execute_unprepared(sql).await?;
        Ok(())
    }
}
