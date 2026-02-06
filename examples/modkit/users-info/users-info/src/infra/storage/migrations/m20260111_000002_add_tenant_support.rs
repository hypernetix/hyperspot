use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let conn = manager.get_connection();
        let root_tenant = modkit_security::constants::DEFAULT_TENANT_ID;

        if backend == sea_orm::DatabaseBackend::MySql {
            let tenant_added = if manager.has_column("users", "tenant_id").await? {
                false
            } else {
                conn.execute_unprepared("ALTER TABLE users ADD COLUMN tenant_id VARCHAR(36);")
                    .await?;
                true
            };

            conn.execute_unprepared(&format!(
                "UPDATE users SET tenant_id = '{root_tenant}' WHERE tenant_id IS NULL;"
            ))
            .await?;

            if tenant_added {
                conn.execute_unprepared(
                    "ALTER TABLE users MODIFY COLUMN tenant_id VARCHAR(36) NOT NULL;",
                )
                .await?;
            }

            if manager.has_index("users", "idx_users_email").await? {
                conn.execute_unprepared("DROP INDEX idx_users_email ON users;")
                    .await?;
            }

            if !manager.has_index("users", "uk_users_tenant_email").await? {
                conn.execute_unprepared(
                    "CREATE UNIQUE INDEX uk_users_tenant_email ON users(tenant_id, email);",
                )
                .await?;
            }

            if !manager.has_index("users", "idx_users_tenant").await? {
                conn.execute_unprepared("CREATE INDEX idx_users_tenant ON users(tenant_id);")
                    .await?;
            }

            Ok(())
        } else {
            let sql = match backend {
                sea_orm::DatabaseBackend::Postgres => format!(
                    r"
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
                    "
                ),
                sea_orm::DatabaseBackend::Sqlite => format!(
                    r"
-- SQLite: Add tenant_id column with default
ALTER TABLE users ADD COLUMN tenant_id TEXT NOT NULL DEFAULT '{root_tenant}';

-- Drop old unique index on email
DROP INDEX IF EXISTS idx_users_email;

-- Create composite unique index on (tenant_id, email)
CREATE UNIQUE INDEX IF NOT EXISTS uk_users_tenant_email ON users(tenant_id, email);

-- Create index on tenant_id for filtering
CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id);
                    "
                ),
                sea_orm::DatabaseBackend::MySql => unreachable!("handled above"),
            };

            conn.execute_unprepared(&sql).await?;
            Ok(())
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let conn = manager.get_connection();

        if backend == sea_orm::DatabaseBackend::MySql {
            if manager.has_index("users", "uk_users_tenant_email").await? {
                conn.execute_unprepared("DROP INDEX uk_users_tenant_email ON users;")
                    .await?;
            }

            if manager.has_index("users", "idx_users_tenant").await? {
                conn.execute_unprepared("DROP INDEX idx_users_tenant ON users;")
                    .await?;
            }

            if !manager.has_index("users", "idx_users_email").await? {
                conn.execute_unprepared("CREATE UNIQUE INDEX idx_users_email ON users(email);")
                    .await?;
            }

            if manager.has_column("users", "tenant_id").await? {
                conn.execute_unprepared("ALTER TABLE users DROP COLUMN tenant_id;")
                    .await?;
            }

            Ok(())
        } else {
            let sql = match backend {
                sea_orm::DatabaseBackend::Postgres => {
                    r"
-- Drop composite unique index
DROP INDEX IF EXISTS uk_users_tenant_email;

-- Drop tenant index
DROP INDEX IF EXISTS idx_users_tenant;

-- Restore old unique index on email
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Drop tenant_id column
ALTER TABLE users DROP COLUMN IF EXISTS tenant_id;
                    "
                }
                sea_orm::DatabaseBackend::Sqlite => {
                    r"
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
                    "
                }
                sea_orm::DatabaseBackend::MySql => unreachable!("handled above"),
            };

            conn.execute_unprepared(sql).await?;
            Ok(())
        }
    }
}
