//! SeaORM entities for OAGW.

pub use link::Entity as LinkEntity;
pub use route::Entity as RouteEntity;

/// Route entity module.
pub mod route {
    use chrono::{DateTime, Utc};
    use modkit_db_macros::Scopable;
    use sea_orm::entity::prelude::*;
    use uuid::Uuid;

    /// Route entity for `outbound_api_route` table.
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Scopable)]
    #[sea_orm(table_name = "outbound_api_route")]
    #[secure(tenant_col = "tenant_id", resource_col = "id", no_owner, no_type)]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub tenant_id: Uuid,
        pub base_url: String,
        pub rate_limit_req_per_min: i32,
        pub auth_type_gts_id: String,
        pub cache_ttl_sec: Option<i32>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::link::Entity")]
        Links,
        #[sea_orm(has_many = "super::route_protocol::Entity")]
        Protocols,
    }

    impl Related<super::link::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Links.def()
        }
    }

    impl Related<super::route_protocol::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Protocols.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Route supported protocol entity module.
pub mod route_protocol {
    use modkit_db_macros::Scopable;
    use sea_orm::entity::prelude::*;
    use uuid::Uuid;

    /// Route protocol entity for `outbound_api_route_supported_protocol` table.
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Scopable)]
    #[sea_orm(table_name = "outbound_api_route_supported_protocol")]
    #[secure(no_tenant, resource_col = "id", no_owner, no_type)]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub route_id: Uuid,
        pub protocol_gts_id: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::route::Entity",
            from = "Column::RouteId",
            to = "super::route::Column::Id"
        )]
        Route,
    }

    impl Related<super::route::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Route.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Link entity module.
pub mod link {
    use chrono::{DateTime, Utc};
    use modkit_db_macros::Scopable;
    use sea_orm::entity::prelude::*;
    use uuid::Uuid;

    /// Link entity for `outbound_api_link` table.
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Scopable)]
    #[sea_orm(table_name = "outbound_api_link")]
    #[secure(tenant_col = "tenant_id", resource_col = "id", no_owner, no_type)]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub tenant_id: Uuid,
        pub secret_ref: Uuid,
        pub route_id: Uuid,
        pub secret_type_gts_id: String,
        pub enabled: bool,
        pub priority: i32,
        pub strategy_gts_id: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::route::Entity",
            from = "Column::RouteId",
            to = "super::route::Column::Id"
        )]
        Route,
    }

    impl Related<super::route::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Route.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
