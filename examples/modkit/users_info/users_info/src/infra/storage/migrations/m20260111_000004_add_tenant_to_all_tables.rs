use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let conn = manager.get_connection();

        if backend == sea_orm::DatabaseBackend::MySql {
            // cities
            if !manager.has_column("cities", "tenant_id").await? {
                conn.execute_unprepared("ALTER TABLE cities ADD COLUMN tenant_id VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';")
                    .await?;
            }
            if !manager.has_index("cities", "idx_cities_tenant").await? {
                conn.execute_unprepared("CREATE INDEX idx_cities_tenant ON cities(tenant_id);")
                    .await?;
            }

            // addresses
            if !manager.has_column("addresses", "tenant_id").await? {
                conn.execute_unprepared("ALTER TABLE addresses ADD COLUMN tenant_id VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';")
                    .await?;
            }
            if !manager
                .has_index("addresses", "idx_addresses_tenant")
                .await?
            {
                conn.execute_unprepared(
                    "CREATE INDEX idx_addresses_tenant ON addresses(tenant_id);",
                )
                .await?;
            }

            Ok(())
        } else {
            let sql = match backend {
                sea_orm::DatabaseBackend::Postgres => {
                    r"
-- Add tenant_id to cities table
ALTER TABLE cities ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_cities_tenant ON cities(tenant_id);

-- Add tenant_id to addresses table
ALTER TABLE addresses ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_addresses_tenant ON addresses(tenant_id);
                    "
                }
                sea_orm::DatabaseBackend::Sqlite => {
                    r"
-- SQLite doesn't support ALTER TABLE ADD COLUMN IF NOT EXISTS, so we need to check first
-- For SQLite, we'll use a different approach with CREATE TABLE IF NOT EXISTS and data migration

-- Add tenant_id to cities table
ALTER TABLE cities ADD COLUMN tenant_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_cities_tenant ON cities(tenant_id);

-- Add tenant_id to addresses table
ALTER TABLE addresses ADD COLUMN tenant_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_addresses_tenant ON addresses(tenant_id);
                    "
                }
                sea_orm::DatabaseBackend::MySql => unreachable!("handled above"),
            };

            conn.execute_unprepared(sql).await?;
            Ok(())
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let conn = manager.get_connection();

        match backend {
            sea_orm::DatabaseBackend::Sqlite => Ok(()),
            sea_orm::DatabaseBackend::MySql => {
                // drop indexes first (if they exist)
                if manager.has_index("cities", "idx_cities_tenant").await? {
                    conn.execute_unprepared("DROP INDEX idx_cities_tenant ON cities;")
                        .await?;
                }
                if manager
                    .has_index("addresses", "idx_addresses_tenant")
                    .await?
                {
                    conn.execute_unprepared("DROP INDEX idx_addresses_tenant ON addresses;")
                        .await?;
                }

                // drop columns (if they exist)
                if manager.has_column("cities", "tenant_id").await? {
                    conn.execute_unprepared("ALTER TABLE cities DROP COLUMN tenant_id;")
                        .await?;
                }
                if manager.has_column("addresses", "tenant_id").await? {
                    conn.execute_unprepared("ALTER TABLE addresses DROP COLUMN tenant_id;")
                        .await?;
                }

                Ok(())
            }
            sea_orm::DatabaseBackend::Postgres => {
                let sql = match backend {
                    sea_orm::DatabaseBackend::Postgres => {
                        r"
ALTER TABLE cities DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE addresses DROP COLUMN IF EXISTS tenant_id;
                        "
                    }
                    sea_orm::DatabaseBackend::MySql | sea_orm::DatabaseBackend::Sqlite => {
                        unreachable!("handled above")
                    }
                };

                conn.execute_unprepared(sql).await?;
                Ok(())
            }
        }
    }
}
