//! Entity to domain model mappers.

use oagw_sdk::{Link, NewLink, NewRoute, Route};

use super::entity::{link, route};

/// Convert route entity to domain model.
impl From<route::Model> for Route {
    fn from(model: route::Model) -> Self {
        Self {
            id: model.id,
            tenant_id: model.tenant_id,
            base_url: model.base_url,
            rate_limit_req_per_min: model.rate_limit_req_per_min,
            auth_type_gts_id: model.auth_type_gts_id,
            cache_ttl_sec: model.cache_ttl_sec,
            supported_protocols: Vec::new(), // Loaded separately
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Convert route with protocols to domain model.
pub fn route_with_protocols(
    model: route::Model,
    protocols: Vec<super::entity::route_protocol::Model>,
) -> Route {
    Route {
        id: model.id,
        tenant_id: model.tenant_id,
        base_url: model.base_url,
        rate_limit_req_per_min: model.rate_limit_req_per_min,
        auth_type_gts_id: model.auth_type_gts_id,
        cache_ttl_sec: model.cache_ttl_sec,
        supported_protocols: protocols.into_iter().map(|p| p.protocol_gts_id).collect(),
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

/// Convert new route to active model.
pub fn new_route_to_active_model(
    new_route: &NewRoute,
    id: uuid::Uuid,
    now: chrono::DateTime<chrono::Utc>,
) -> route::ActiveModel {
    use sea_orm::ActiveValue::Set;

    route::ActiveModel {
        id: Set(id),
        tenant_id: Set(new_route.tenant_id),
        base_url: Set(new_route.base_url.clone()),
        rate_limit_req_per_min: Set(new_route.rate_limit_req_per_min.unwrap_or(1000)),
        auth_type_gts_id: Set(new_route.auth_type_gts_id.clone()),
        cache_ttl_sec: Set(new_route.cache_ttl_sec),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

/// Convert link entity to domain model.
impl From<link::Model> for Link {
    fn from(model: link::Model) -> Self {
        Self {
            id: model.id,
            tenant_id: model.tenant_id,
            secret_ref: model.secret_ref,
            route_id: model.route_id,
            secret_type_gts_id: model.secret_type_gts_id,
            enabled: model.enabled,
            priority: model.priority,
            strategy_gts_id: model.strategy_gts_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Convert new link to active model.
pub fn new_link_to_active_model(
    new_link: &NewLink,
    id: uuid::Uuid,
    now: chrono::DateTime<chrono::Utc>,
) -> link::ActiveModel {
    use sea_orm::ActiveValue::Set;

    link::ActiveModel {
        id: Set(id),
        tenant_id: Set(new_link.tenant_id),
        secret_ref: Set(new_link.secret_ref),
        route_id: Set(new_link.route_id),
        secret_type_gts_id: Set(new_link.secret_type_gts_id.clone()),
        enabled: Set(new_link.enabled.unwrap_or(true)),
        priority: Set(new_link.priority.unwrap_or(0)),
        strategy_gts_id: Set(new_link.strategy_gts_id.clone()),
        created_at: Set(now),
        updated_at: Set(now),
    }
}
