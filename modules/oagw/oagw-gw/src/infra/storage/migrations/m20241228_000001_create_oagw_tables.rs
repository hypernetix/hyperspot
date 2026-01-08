//! Initial migration for OAGW tables.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create outbound_api_route table
        manager
            .create_table(
                Table::create()
                    .table(OutboundApiRoute::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OutboundApiRoute::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OutboundApiRoute::TenantId).uuid().not_null())
                    .col(ColumnDef::new(OutboundApiRoute::BaseUrl).text().not_null())
                    .col(
                        ColumnDef::new(OutboundApiRoute::RateLimitReqPerMin)
                            .integer()
                            .not_null()
                            .default(1000),
                    )
                    .col(
                        ColumnDef::new(OutboundApiRoute::AuthTypeGtsId)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(OutboundApiRoute::CacheTtlSec).integer())
                    .col(
                        ColumnDef::new(OutboundApiRoute::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OutboundApiRoute::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on tenant_id
        manager
            .create_index(
                Index::create()
                    .name("idx_outbound_api_route_tenant")
                    .table(OutboundApiRoute::Table)
                    .col(OutboundApiRoute::TenantId)
                    .to_owned(),
            )
            .await?;

        // Create outbound_api_route_supported_protocol table
        manager
            .create_table(
                Table::create()
                    .table(OutboundApiRouteSupportedProtocol::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OutboundApiRouteSupportedProtocol::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OutboundApiRouteSupportedProtocol::RouteId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OutboundApiRouteSupportedProtocol::ProtocolGtsId)
                            .text()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                OutboundApiRouteSupportedProtocol::Table,
                                OutboundApiRouteSupportedProtocol::RouteId,
                            )
                            .to(OutboundApiRoute::Table, OutboundApiRoute::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create outbound_api_link table
        manager
            .create_table(
                Table::create()
                    .table(OutboundApiLink::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OutboundApiLink::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OutboundApiLink::TenantId).uuid().not_null())
                    .col(ColumnDef::new(OutboundApiLink::SecretRef).uuid().not_null())
                    .col(ColumnDef::new(OutboundApiLink::RouteId).uuid().not_null())
                    .col(
                        ColumnDef::new(OutboundApiLink::SecretTypeGtsId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OutboundApiLink::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(OutboundApiLink::Priority)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(OutboundApiLink::StrategyGtsId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OutboundApiLink::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OutboundApiLink::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(OutboundApiLink::Table, OutboundApiLink::RouteId)
                            .to(OutboundApiRoute::Table, OutboundApiRoute::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes on link table
        manager
            .create_index(
                Index::create()
                    .name("idx_outbound_api_link_tenant")
                    .table(OutboundApiLink::Table)
                    .col(OutboundApiLink::TenantId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_outbound_api_link_route")
                    .table(OutboundApiLink::Table)
                    .col(OutboundApiLink::RouteId)
                    .to_owned(),
            )
            .await?;

        // TODO(v2): Create outbound_api_route_limits table
        // TODO(v2): Create outbound_api_audit_log table

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OutboundApiLink::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(OutboundApiRouteSupportedProtocol::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(OutboundApiRoute::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum OutboundApiRoute {
    Table,
    Id,
    TenantId,
    BaseUrl,
    RateLimitReqPerMin,
    AuthTypeGtsId,
    CacheTtlSec,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum OutboundApiRouteSupportedProtocol {
    Table,
    Id,
    RouteId,
    ProtocolGtsId,
}

#[derive(Iden)]
enum OutboundApiLink {
    Table,
    Id,
    TenantId,
    SecretRef,
    RouteId,
    SecretTypeGtsId,
    Enabled,
    Priority,
    StrategyGtsId,
    CreatedAt,
    UpdatedAt,
}
