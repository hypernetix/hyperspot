//! SeaORM repository implementation for links.

use async_trait::async_trait;
use modkit_db::secure::SecureConn;
use modkit_odata::{ODataQuery, Page, PageInfo};
use modkit_security::AccessScope;
use oagw_sdk::{Link, LinkPatch, NewLink};
use sea_orm::{ActiveValue, ColumnTrait, Condition, Order};
use uuid::Uuid;

use super::entity::link;
use super::mapper::new_link_to_active_model;
use crate::domain::repo::LinkRepository;

/// SeaORM implementation of LinkRepository.
pub struct SeaOrmLinkRepository {
    conn: SecureConn,
}

impl SeaOrmLinkRepository {
    /// Create a new repository instance.
    pub fn new(conn: SecureConn) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl LinkRepository for SeaOrmLinkRepository {
    async fn find_by_id(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<Option<Link>> {
        let link_opt = self
            .conn
            .find_by_id::<link::Entity>(scope, id)?
            .one(self.conn.conn())
            .await?;

        Ok(link_opt.map(Link::from))
    }

    async fn find_enabled_by_route(
        &self,
        scope: &AccessScope,
        route_id: Uuid,
    ) -> anyhow::Result<Vec<Link>> {
        let base_query = self.conn.find::<link::Entity>(scope);

        let filter_cond = Condition::all()
            .add(link::Column::RouteId.eq(route_id))
            .add(link::Column::Enabled.eq(true));

        let links: Vec<link::Model> = base_query
            .filter(filter_cond)
            .order_by(link::Column::Priority, Order::Asc)
            .all(self.conn.conn())
            .await?;

        Ok(links.into_iter().map(Link::from).collect())
    }

    async fn list_page(
        &self,
        scope: &AccessScope,
        query: ODataQuery,
    ) -> anyhow::Result<Page<Link>> {
        // TODO(v2): Implement proper OData filtering with paginate_odata
        // For v1, simple pagination without filters

        let limit = query.limit.unwrap_or(25).min(1000);

        let base_query = self.conn.find::<link::Entity>(scope);

        let links: Vec<link::Model> = base_query
            .order_by(link::Column::CreatedAt, Order::Desc)
            .limit(limit + 1)
            .all(self.conn.conn())
            .await?;

        let has_more = links.len() > limit as usize;
        let items: Vec<_> = links
            .into_iter()
            .take(limit as usize)
            .map(Link::from)
            .collect();

        Ok(Page {
            items,
            page_info: PageInfo {
                next_cursor: if has_more {
                    Some("next".to_string())
                } else {
                    None
                }, // TODO(v2): Implement proper cursor
                prev_cursor: None,
                limit,
            },
        })
    }

    async fn insert(&self, scope: &AccessScope, new_link: NewLink) -> anyhow::Result<Link> {
        let id = new_link.id.unwrap_or_else(Uuid::now_v7);
        let now = chrono::Utc::now();

        let active_model = new_link_to_active_model(&new_link, id, now);
        let model = self
            .conn
            .insert::<link::Entity>(scope, active_model)
            .await?;

        Ok(Link::from(model))
    }

    async fn update(
        &self,
        scope: &AccessScope,
        id: Uuid,
        patch: LinkPatch,
    ) -> anyhow::Result<Link> {
        let now = chrono::Utc::now();

        // Build active model with only changed fields
        let mut active_model = link::ActiveModel {
            id: ActiveValue::Unchanged(id),
            updated_at: ActiveValue::Set(now),
            ..Default::default()
        };

        if let Some(secret_ref) = patch.secret_ref {
            active_model.secret_ref = ActiveValue::Set(secret_ref);
        }
        if let Some(secret_type) = patch.secret_type_gts_id {
            active_model.secret_type_gts_id = ActiveValue::Set(secret_type);
        }
        if let Some(enabled) = patch.enabled {
            active_model.enabled = ActiveValue::Set(enabled);
        }
        if let Some(priority) = patch.priority {
            active_model.priority = ActiveValue::Set(priority);
        }
        if let Some(strategy) = patch.strategy_gts_id {
            active_model.strategy_gts_id = ActiveValue::Set(strategy);
        }

        let model = self
            .conn
            .update_with_ctx::<link::Entity>(scope, id, active_model)
            .await?;

        Ok(Link::from(model))
    }

    async fn delete(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<bool> {
        Ok(self.conn.delete_by_id::<link::Entity>(scope, id).await?)
    }

    async fn exists(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<bool> {
        let link = self
            .conn
            .find_by_id::<link::Entity>(scope, id)?
            .one(self.conn.conn())
            .await?;
        Ok(link.is_some())
    }
}
