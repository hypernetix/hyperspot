//! `SeaORM` repository implementation for routes.

use async_trait::async_trait;
use modkit_db::secure::SecureConn;
use modkit_odata::{ODataQuery, Page, PageInfo};
use modkit_security::AccessScope;
use oagw_sdk::{NewRoute, Route, RoutePatch};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, Order, QueryFilter};
use uuid::Uuid;

use super::entity::{route, route_protocol};
use super::mapper::{new_route_to_active_model, route_with_protocols};
use crate::domain::repo::RouteRepository;

/// `SeaORM` implementation of `RouteRepository`.
pub struct SeaOrmRouteRepository {
    conn: SecureConn,
}

impl SeaOrmRouteRepository {
    /// Create a new repository instance.
    #[must_use]
    pub fn new(conn: SecureConn) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl RouteRepository for SeaOrmRouteRepository {
    async fn find_by_id(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<Option<Route>> {
        let route_opt = self
            .conn
            .find_by_id::<route::Entity>(scope, id)?
            .one(self.conn.conn())
            .await?;

        let Some(route_model) = route_opt else {
            return Ok(None);
        };

        // Load associated protocols
        // Note: route_protocol is a join table without tenant_id - access is controlled via route
        #[allow(clippy::disallowed_methods)]
        let protocols = route_protocol::Entity::find()
            .filter(route_protocol::Column::RouteId.eq(id))
            .all(self.conn.conn())
            .await?;

        Ok(Some(route_with_protocols(route_model, protocols)))
    }

    async fn list_page(
        &self,
        scope: &AccessScope,
        query: ODataQuery,
    ) -> anyhow::Result<Page<Route>> {
        // TODO(v2): Implement proper OData filtering with paginate_odata
        // For v1, simple pagination without filters

        let limit = query.limit.unwrap_or(25).min(1000);

        let base_query = self.conn.find::<route::Entity>(scope);

        let routes: Vec<route::Model> = base_query
            .order_by(route::Column::CreatedAt, Order::Desc)
            .limit(limit + 1) // Fetch one extra to check for more
            .all(self.conn.conn())
            .await?;

        let has_more = routes.len() > limit as usize;
        let items: Vec<_> = routes
            .into_iter()
            .take(limit as usize)
            .map(Route::from)
            .collect();

        // TODO(v2): Load protocols for each route in batch
        // For v1, protocols are loaded separately when needed

        Ok(Page {
            items,
            page_info: PageInfo {
                next_cursor: if has_more {
                    Some("next".to_owned())
                } else {
                    None
                }, // TODO(v2): Implement proper cursor
                prev_cursor: None,
                limit,
            },
        })
    }

    async fn insert(&self, scope: &AccessScope, new_route: NewRoute) -> anyhow::Result<Route> {
        let id = new_route.id.unwrap_or_else(Uuid::now_v7);
        let now = chrono::Utc::now();

        let active_model = new_route_to_active_model(&new_route, id, now);
        let model = self
            .conn
            .insert::<route::Entity>(scope, active_model)
            .await?;

        // Insert supported protocols
        for protocol_gts_id in &new_route.supported_protocols {
            let protocol_model = route_protocol::ActiveModel {
                id: ActiveValue::Set(Uuid::now_v7()),
                route_id: ActiveValue::Set(id),
                protocol_gts_id: ActiveValue::Set(protocol_gts_id.clone()),
            };
            protocol_model.insert(self.conn.conn()).await?;
        }

        Ok(route_with_protocols(
            model,
            new_route
                .supported_protocols
                .into_iter()
                .map(|protocol_gts_id| route_protocol::Model {
                    id: Uuid::nil(), // Not needed for conversion
                    route_id: id,
                    protocol_gts_id,
                })
                .collect(),
        ))
    }

    async fn update(
        &self,
        scope: &AccessScope,
        id: Uuid,
        patch: RoutePatch,
    ) -> anyhow::Result<Route> {
        let now = chrono::Utc::now();

        // Build active model with only changed fields
        let mut active_model = route::ActiveModel {
            id: ActiveValue::Unchanged(id),
            updated_at: ActiveValue::Set(now),
            ..Default::default()
        };

        if let Some(base_url) = patch.base_url {
            active_model.base_url = ActiveValue::Set(base_url);
        }
        if let Some(rate_limit) = patch.rate_limit_req_per_min {
            active_model.rate_limit_req_per_min = ActiveValue::Set(rate_limit);
        }
        if let Some(auth_type) = patch.auth_type_gts_id {
            active_model.auth_type_gts_id = ActiveValue::Set(auth_type);
        }
        if let Some(cache_ttl) = patch.cache_ttl_sec {
            active_model.cache_ttl_sec = ActiveValue::Set(cache_ttl);
        }

        let model = self
            .conn
            .update_with_ctx::<route::Entity>(scope, id, active_model)
            .await?;

        // Update protocols if provided
        if let Some(protocols) = patch.supported_protocols {
            // Delete existing protocols
            // Note: route_protocol is a join table without tenant_id - access is controlled via route
            #[allow(clippy::disallowed_methods)]
            route_protocol::Entity::delete_many()
                .filter(route_protocol::Column::RouteId.eq(id))
                .exec(self.conn.conn())
                .await?;

            // Insert new protocols
            for protocol_gts_id in &protocols {
                let protocol_model = route_protocol::ActiveModel {
                    id: ActiveValue::Set(Uuid::now_v7()),
                    route_id: ActiveValue::Set(id),
                    protocol_gts_id: ActiveValue::Set(protocol_gts_id.clone()),
                };
                protocol_model.insert(self.conn.conn()).await?;
            }

            return Ok(route_with_protocols(
                model,
                protocols
                    .into_iter()
                    .map(|protocol_gts_id| route_protocol::Model {
                        id: Uuid::nil(),
                        route_id: id,
                        protocol_gts_id,
                    })
                    .collect(),
            ));
        }

        // Load current protocols
        // Note: route_protocol is a join table without tenant_id - access is controlled via route
        #[allow(clippy::disallowed_methods)]
        let protocols = route_protocol::Entity::find()
            .filter(route_protocol::Column::RouteId.eq(id))
            .all(self.conn.conn())
            .await?;

        Ok(route_with_protocols(model, protocols))
    }

    async fn delete(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<bool> {
        // Delete associated protocols first (cascade)
        // Note: route_protocol is a join table without tenant_id - access is controlled via route
        #[allow(clippy::disallowed_methods)]
        route_protocol::Entity::delete_many()
            .filter(route_protocol::Column::RouteId.eq(id))
            .exec(self.conn.conn())
            .await?;

        Ok(self.conn.delete_by_id::<route::Entity>(scope, id).await?)
    }

    async fn exists(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<bool> {
        let route = self
            .conn
            .find_by_id::<route::Entity>(scope, id)?
            .one(self.conn.conn())
            .await?;
        Ok(route.is_some())
    }
}
