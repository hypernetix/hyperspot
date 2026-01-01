use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let conn = manager.get_connection();
        let root_tenant = modkit_security::constants::ROOT_TENANT_ID;

        let sql = match backend {
            sea_orm::DatabaseBackend::Postgres => {
                format!(
                    r#"
-- Add tenant_id column
ALTER TABLE users ADD COLUMN IF NOT EXISTS tenant_id UUID;

-- Backfill existing rows with root tenant
UPDATE users SET tenant_id = '{root_tenant}' WHERE tenant_id IS NULL;

-- Make tenant_id NOT NULL
ALTER TABLE users ALTER COLUMN tenant_id SET NOT NULL;

-- Drop old unique index on email
DROP INDEX IF EXISTS idx_users_email;

-- Create composite unique index on (tenant_id, email)
CREATE UNIQUE INDEX IF NOT EXISTS uk_users_tenant_email ON users(tenant_id, email);

-- Create index on tenant_id for filtering
CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id);
                "#
                )
            }
            sea_orm::DatabaseBackend::MySql => {
                format!(
                    r#"
-- Add tenant_id column
ALTER TABLE users ADD COLUMN tenant_id VARCHAR(36);

-- Backfill existing rows with root tenant
UPDATE users SET tenant_id = '{root_tenant}' WHERE tenant_id IS NULL;

-- Make tenant_id NOT NULL
ALTER TABLE users MODIFY COLUMN tenant_id VARCHAR(36) NOT NULL;

-- Drop old unique index on email
DROP INDEX idx_users_email ON users;

-- Create composite unique index on (tenant_id, email)
CREATE UNIQUE INDEX uk_users_tenant_email ON users(tenant_id, email);

-- Create index on tenant_id for filtering
CREATE INDEX idx_users_tenant ON users(tenant_id);
                "#
                )
            }
            sea_orm::DatabaseBackend::Sqlite => {
                format!(
                    r#"
-- SQLite: Add tenant_id column with default
ALTER TABLE users ADD COLUMN tenant_id TEXT NOT NULL DEFAULT '{root_tenant}';

-- Drop old unique index on email
DROP INDEX IF EXISTS idx_users_email;

-- Create composite unique index on (tenant_id, email)
CREATE UNIQUE INDEX IF NOT EXISTS uk_users_tenant_email ON users(tenant_id, email);

-- Create index on tenant_id for filtering
CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id);
                "#
                )
            }
        };

        conn.execute_unprepared(&sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let conn = manager.get_connection();

        let sql = match backend {
            sea_orm::DatabaseBackend::Postgres => {
                r#"
-- Drop composite unique index
DROP INDEX IF EXISTS uk_users_tenant_email;

-- Drop tenant index
DROP INDEX IF EXISTS idx_users_tenant;

-- Restore old unique index on email
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Drop tenant_id column
ALTER TABLE users DROP COLUMN IF EXISTS tenant_id;
                "#
            }
            sea_orm::DatabaseBackend::MySql => {
                r#"
-- Drop composite unique index
DROP INDEX uk_users_tenant_email ON users;

-- Drop tenant index
DROP INDEX idx_users_tenant ON users;

-- Restore old unique index on email
CREATE UNIQUE INDEX idx_users_email ON users(email);

-- Drop tenant_id column
ALTER TABLE users DROP COLUMN tenant_id;
                "#
            }
            sea_orm::DatabaseBackend::Sqlite => {
                r#"
-- SQLite: Cannot drop columns, need to recreate table
-- Drop indexes first
DROP INDEX IF EXISTS uk_users_tenant_email;
DROP INDEX IF EXISTS idx_users_tenant;

-- Recreate table without tenant_id
CREATE TABLE users_new (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT INTO users_new (id, email, display_name, created_at, updated_at)
SELECT id, email, display_name, created_at, updated_at FROM users;

DROP TABLE users;
ALTER TABLE users_new RENAME TO users;

-- Restore old unique index on email
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email);
                "#
            }
        };

        conn.execute_unprepared(sql).await?;
        Ok(())
    }
}
