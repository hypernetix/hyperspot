use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use sea_orm::DatabaseBackend as DB;

        let backend = manager.get_database_backend();
        let users = Users::Table;
        let tenant_id = Users::TenantId;
        let email = Users::Email;

        let idx_old_email = "idx_users_email";
        let uk_tenant_email = "uk_users_tenant_email";
        let idx_tenant = "idx_users_tenant";

        let root_tenant = modkit_security::constants::ROOT_TENANT_ID;

        // 1) Add tenant_id column if it doesn't exist
        if !manager
            .has_column(users.to_string(), tenant_id.to_string())
            .await?
        {
            match backend {
                DB::Postgres | DB::MySql => {
                    // Step 1: Add column as NULL to allow backfill
                    manager
                        .alter_table(
                            Table::alter()
                                .table(users)
                                .add_column(tenant_col_def(backend, tenant_id).null())
                                .to_owned(),
                        )
                        .await?;

                    // Step 2: Backfill all rows with the root tenant UUID
                    let upd = match backend {
                        DB::Postgres => format!(
                            "UPDATE \"users\" SET \"tenant_id\"='{}' WHERE \"tenant_id\" IS NULL",
                            root_tenant
                        ),
                        DB::MySql => format!(
                            "UPDATE `users` SET `tenant_id`='{}' WHERE `tenant_id` IS NULL",
                            root_tenant
                        ),
                        _ => unreachable!(),
                    };
                    manager.get_connection().execute_unprepared(&upd).await?;

                    // Step 3: Set NOT NULL constraint
                    manager
                        .alter_table(
                            Table::alter()
                                .table(users)
                                .modify_column(tenant_col_def(backend, tenant_id).not_null())
                                .to_owned(),
                        )
                        .await?;
                }
                DB::Sqlite => {
                    // SQLite cannot modify columns; add directly with NOT NULL + DEFAULT
                    let sql = format!(
                        "ALTER TABLE \"users\" ADD COLUMN \"tenant_id\" TEXT NOT NULL DEFAULT '{}'",
                        root_tenant
                    );
                    manager.get_connection().execute_unprepared(&sql).await?;
                }
            }
        }

        // 2) Drop the old global unique index on email (if it exists)
        if manager.has_index(users.to_string(), idx_old_email).await? {
            manager
                .drop_index(Index::drop().name(idx_old_email).table(users).to_owned())
                .await?;
        }

        // 3) Create unique index on (tenant_id, email)
        if !manager
            .has_index(users.to_string(), uk_tenant_email)
            .await?
        {
            match backend {
                DB::Postgres | DB::MySql => {
                    manager
                        .create_index(
                            Index::create()
                                .name(uk_tenant_email)
                                .table(users)
                                .col(tenant_id)
                                .col(email)
                                .unique()
                                .to_owned(),
                        )
                        .await?;
                }
                DB::Sqlite => {
                    let sql = format!(
                        "CREATE UNIQUE INDEX IF NOT EXISTS \"{}\" ON \"users\" (\"tenant_id\", \"email\")",
                        uk_tenant_email
                    );
                    manager.get_connection().execute_unprepared(&sql).await?;
                }
            }
        }

        // 4) Create regular index on tenant_id for filtering
        if !manager.has_index(users.to_string(), idx_tenant).await? {
            match backend {
                DB::Postgres | DB::MySql => {
                    manager
                        .create_index(
                            Index::create()
                                .name(idx_tenant)
                                .table(users)
                                .col(tenant_id)
                                .to_owned(),
                        )
                        .await?;
                }
                DB::Sqlite => {
                    let sql = format!(
                        "CREATE INDEX IF NOT EXISTS \"{}\" ON \"users\" (\"tenant_id\")",
                        idx_tenant
                    );
                    manager.get_connection().execute_unprepared(&sql).await?;
                }
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use sea_orm::DatabaseBackend as DB;

        let backend = manager.get_database_backend();
        let users = Users::Table;
        let tenant_id = Users::TenantId;
        let email = Users::Email;

        let idx_old_email = "idx_users_email";
        let uk_tenant_email = "uk_users_tenant_email";
        let idx_tenant = "idx_users_tenant";

        // 1) Drop the composite unique index
        if manager
            .has_index(users.to_string(), uk_tenant_email)
            .await?
        {
            manager
                .drop_index(Index::drop().name(uk_tenant_email).table(users).to_owned())
                .await?;
        }

        // 2) Restore the old unique index on email
        if !manager.has_index(users.to_string(), idx_old_email).await? {
            match backend {
                DB::Postgres | DB::MySql => {
                    manager
                        .create_index(
                            Index::create()
                                .name(idx_old_email)
                                .table(users)
                                .col(email)
                                .unique()
                                .to_owned(),
                        )
                        .await?;
                }
                DB::Sqlite => {
                    let sql = format!(
                        "CREATE UNIQUE INDEX IF NOT EXISTS \"{}\" ON \"users\" (\"email\")",
                        idx_old_email
                    );
                    manager.get_connection().execute_unprepared(&sql).await?;
                }
            }
        }

        // 3) Drop the tenant_id index
        if manager.has_index(users.to_string(), idx_tenant).await? {
            manager
                .drop_index(Index::drop().name(idx_tenant).table(users).to_owned())
                .await?;
        }

        // 4) Drop the tenant_id column
        manager
            .alter_table(
                Table::alter()
                    .table(users)
                    .drop_column(tenant_id)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden, Copy, Clone)]
enum Users {
    #[sea_orm(iden = "users")]
    Table,
    Email,
    #[sea_orm(iden = "tenant_id")]
    TenantId,
}

/// Helper to generate the correct column type for Postgres/MySQL
/// - Postgres: native UUID
/// - MySQL: VARCHAR(36)
fn tenant_col_def(backend: sea_orm::DatabaseBackend, col: Users) -> ColumnDef {
    match backend {
        sea_orm::DatabaseBackend::Postgres => ColumnDef::new(col).uuid().to_owned(),
        sea_orm::DatabaseBackend::MySql => ColumnDef::new(col).string_len(36).to_owned(),
        _ => unreachable!("tenant_col_def is only used for Postgres/MySQL paths"),
    }
}
