//! REST API route definitions - OpenAPI and Axum routing.
//!
//! ## Architecture
//!
//! This module defines REST routes with OpenAPI metadata organized by resource:
//! - `users` - User endpoints (5: list, get, create, update, delete)
//! - `cities` - City endpoints (5: list, get, create, update, delete)
//! - `languages` - Language endpoints (5: list, get, create, update, delete)
//! - `addresses` - Address endpoints (3: get, upsert, delete)
//! - `relations` - User-Language relation endpoints (3: list, assign, remove)
//! - `events` - SSE event stream (1: user events)
//!
//! ## OData Integration
//!
//! List endpoints support OData query parameters via SDK filter schemas:
//! - `$filter` - Type-safe filtering using `user_info_sdk::odata::*FilterField`
//! - `$orderby` - Sorting on filterable fields
//! - `$select` - Field projection for response optimization
//! - Pagination via cursor-based `limit` and `cursor` params
//!
//! ## Layering
//!
//! Routes orchestrate but don't contain business logic:
//! - Delegate to `handlers::*` for request processing
//! - Handlers call `domain::service::Service` for business operations
//! - Use `dto::*` types for request/response serialization

use crate::api::rest::{dto, handlers};
use crate::domain::service::Service;
use axum::Router;
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature};
use modkit::api::OpenApiRegistry;
use std::sync::Arc;

mod addresses;
mod cities;
mod events;
mod languages;
mod relations;
mod users;

// Shared authorization enums and types

pub(super) enum Resource {
    Users,
    Cities,
    Languages,
    Addresses,
    UserLanguages,
}

pub(super) enum Action {
    Read,
    Delete,
    Update,
    Create,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        match self {
            Resource::Users => "users",
            Resource::Cities => "cities",
            Resource::Languages => "languages",
            Resource::Addresses => "addresses",
            Resource::UserLanguages => "user_languages",
        }
    }
}

impl AuthReqResource for Resource {}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Delete => "delete",
            Action::Update => "update",
            Action::Create => "create",
        }
    }
}

impl AuthReqAction for Action {}

pub(super) struct License;

impl AsRef<str> for License {
    fn as_ref(&self) -> &'static str {
        "gts.x.core.lic.feat.v1~x.core.global.base.v1"
    }
}

impl LicenseFeature for License {}

/// Register all routes for the users_info module
#[allow(clippy::needless_pass_by_value)]
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> Router {
    router = users::register_user_routes(router, openapi);
    router = cities::register_city_routes(router, openapi);
    router = languages::register_language_routes(router, openapi);
    router = addresses::register_address_routes(router, openapi);
    router = relations::register_relation_routes(router, openapi);

    router = router.layer(axum::Extension(service));

    router
}

/// Register SSE route for user events
pub fn register_users_sse_route<S>(
    router: Router<S>,
    openapi: &dyn OpenApiRegistry,
    sse: modkit::SseBroadcaster<dto::UserEvent>,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    events::register_sse_route(router, openapi, sse)
}
