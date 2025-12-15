//! Migration helpers for secure multi-tenant schemas.
//!
//! This module provides documentation and patterns for adding security-related
//! columns and constraints to your migrations.
//!
//! # Usage
//!
//! In your migration code (where `sea_orm_migration` is available), use these patterns:
//!
//! ```ignore
//! use sea_orm_migration::prelude::*;
//!
//! #[derive(Iden)]
//! enum Users {
//!     Table,
//!     Id,
//!     TenantId,
//!     OwnerId,
//!     Email,
//!     CreatedBy,
//!     UpdatedBy,
//!     CreatedAt,
//!     UpdatedAt,
//! }
//!
//! // Create table with security columns
//! Table::create()
//!     .table(Users::Table)
//!     .if_not_exists()
//!     .col(ColumnDef::new(Users::Id).uuid().primary_key())
//!     // Tenant column (required for multi-tenant entities)
//!     .col(ColumnDef::new(Users::TenantId).uuid().not_null())
//!     // Owner column (who created this resource)
//!     .col(ColumnDef::new(Users::OwnerId).uuid().not_null())
//!     // Audit trail columns
//!     .col(ColumnDef::new(Users::CreatedBy).uuid().not_null())
//!     .col(ColumnDef::new(Users::UpdatedBy).uuid())
//!     .col(
//!         ColumnDef::new(Users::CreatedAt)
//!             .timestamp_with_time_zone()
//!             .not_null()
//!             .default(Expr::current_timestamp()),
//!     )
//!     .col(ColumnDef::new(Users::UpdatedAt).timestamp_with_time_zone())
//!     // Business columns
//!     .col(ColumnDef::new(Users::Email).string().not_null())
//!     // Tenant index (critical for query performance)
//!     .index(
//!         Index::create()
//!             .name("idx_users_tenant")
//!             .table(Users::Table)
//!             .col(Users::TenantId)
//!             .to_owned(),
//!     )
//!     // Unique constraint within tenant
//!     .index(
//!         Index::create()
//!             .name("uk_users_tenant_email")
//!             .table(Users::Table)
//!             .col(Users::TenantId)
//!             .col(Users::Email)
//!             .unique()
//!             .to_owned(),
//!     )
//!     .to_owned()
//! ```
//!
//! # Security Column Patterns
//!
//! ## Tenant ID Column
//!
//! Required for all multi-tenant entities:
//!
//! ```ignore
//! .col(ColumnDef::new(TenantId).uuid().not_null())
//! .index(
//!     Index::create()
//!         .name("idx_{table}_tenant")
//!         .table(Table)
//!         .col(TenantId)
//! )
//! ```
//!
//! ## Owner ID Column
//!
//! Tracks who created the resource:
//!
//! ```ignore
//! .col(ColumnDef::new(OwnerId).uuid().not_null())
//! ```
//!
//! ## Audit Trail Columns
//!
//! Full audit trail with timestamps:
//!
//! ```ignore
//! .col(ColumnDef::new(CreatedBy).uuid().not_null())
//! .col(ColumnDef::new(UpdatedBy).uuid())
//! .col(
//!     ColumnDef::new(CreatedAt)
//!         .timestamp_with_time_zone()
//!         .not_null()
//!         .default(Expr::current_timestamp())
//! )
//! .col(ColumnDef::new(UpdatedAt).timestamp_with_time_zone())
//! ```
//!
//! ## Unique Constraints Within Tenant
//!
//! Ensure uniqueness per tenant (not globally):
//!
//! ```ignore
//! .index(
//!     Index::create()
//!         .name("uk_{table}_tenant_{col}")
//!         .table(Table)
//!         .col(TenantId)
//!         .col(Column)
//!         .unique()
//! )
//! ```
//!
//! ## Composite Indexes for Common Queries
//!
//! Optimize queries that filter by tenant + another column:
//!
//! ```ignore
//! .index(
//!     Index::create()
//!         .name("idx_{table}_tenant_status")
//!         .table(Table)
//!         .col(TenantId)
//!         .col(Status)
//! )
//! ```
//!
//! # Global (Non-Tenant) Entities
//!
//! For global entities (no tenant isolation), omit the `tenant_id` column
//! but still include audit fields:
//!
//! ```ignore
//! Table::create()
//!     .table(GlobalSettings::Table)
//!     .col(ColumnDef::new(GlobalSettings::Id).uuid().primary_key())
//!     // No tenant_id column
//!     .col(ColumnDef::new(GlobalSettings::CreatedBy).uuid().not_null())
//!     .col(ColumnDef::new(GlobalSettings::UpdatedBy).uuid())
//!     .col(ColumnDef::new(GlobalSettings::CreatedAt).timestamp_with_time_zone().not_null())
//!     .col(ColumnDef::new(GlobalSettings::UpdatedAt).timestamp_with_time_zone())
//!     .to_owned()
//! ```
//!
//! # Best Practices
//!
//! 1. **Always index `tenant_id`** - Critical for query performance
//! 2. **Use composite indexes** - `tenant_id` should be the first column
//! 3. **Unique constraints include `tenant_id`** - Prevents cross-tenant collisions
//! 4. **Set NOT NULL where appropriate** - Enforce data integrity
//! 5. **Use default for `created_at`** - Automatic timestamp
//! 6. **UUID for all IDs** - Consistent type across security columns

// Note: Migration helper traits could be added here if needed, but keeping
// the module as documentation-only is simpler and avoids dependency issues.
