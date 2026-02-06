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
                r"
-- Create cities table
CREATE TABLE IF NOT EXISTS cities (
    id UUID PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL,
    country VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cities_name ON cities(name);

-- Create addresses table (1:1 with users, N:1 with cities)
CREATE TABLE IF NOT EXISTS addresses (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL,
    city_id UUID NOT NULL,
    street VARCHAR(255) NOT NULL,
    postal_code VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_addresses_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_addresses_city FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_addresses_user ON addresses(user_id);
CREATE INDEX IF NOT EXISTS idx_addresses_city ON addresses(city_id);
                "
            }
            sea_orm::DatabaseBackend::MySql => {
                r"
-- Create cities table
CREATE TABLE IF NOT EXISTS cities (
    id VARCHAR(36) PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL,
    country VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    INDEX idx_cities_name (name)
);

-- Create addresses table (1:1 with users, N:1 with cities)
CREATE TABLE IF NOT EXISTS addresses (
    id VARCHAR(36) PRIMARY KEY NOT NULL,
    user_id VARCHAR(36) NOT NULL,
    city_id VARCHAR(36) NOT NULL,
    street VARCHAR(255) NOT NULL,
    postal_code VARCHAR(50) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    UNIQUE KEY uk_addresses_user (user_id),
    INDEX idx_addresses_city (city_id),
    CONSTRAINT fk_addresses_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_addresses_city FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE RESTRICT
);
                "
            }
            sea_orm::DatabaseBackend::Sqlite => {
                r"
-- Create cities table
CREATE TABLE IF NOT EXISTS cities (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    country TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cities_name ON cities(name);

-- Create addresses table (1:1 with users, N:1 with cities)
CREATE TABLE IF NOT EXISTS addresses (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    city_id TEXT NOT NULL,
    street TEXT NOT NULL,
    postal_code TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_addresses_user ON addresses(user_id);
CREATE INDEX IF NOT EXISTS idx_addresses_city ON addresses(city_id);
                "
            }
        };

        conn.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        let sql = r"
DROP TABLE IF EXISTS addresses;
DROP TABLE IF EXISTS cities;
        ";

        conn.execute_unprepared(sql).await?;
        Ok(())
    }
}
