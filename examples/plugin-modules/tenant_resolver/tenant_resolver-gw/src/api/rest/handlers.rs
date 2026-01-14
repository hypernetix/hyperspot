//! REST API handlers for tenant resolver gateway.

use axum::extract::{Path, Query};
use axum::Extension;
use modkit::api::odata::OData;
use modkit::api::prelude::*;
use modkit::api::select::page_to_projected_json;
use modkit_security::SecurityContext;
use std::sync::Arc;

use crate::api::rest::dto::{
    GetChildrenQuery, GetChildrenResponseDto, GetParentsQuery, GetParentsResponseDto,
    ListTenantsQuery, TenantDto,
};
use crate::domain::service::Service;

/// GET /tenant-resolver/v1/root
#[tracing::instrument(skip_all)]
pub async fn get_root_tenant(
    Extension(sec): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
) -> ApiResult<Json<TenantDto>> {
    let tenant = svc.get_root_tenant(&sec).await?;
    Ok(Json(tenant.into()))
}

/// GET /tenant-resolver/v1/tenants
#[tracing::instrument(skip_all)]
pub async fn list_tenants(
    Extension(sec): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Query(query): Query<ListTenantsQuery>,
    OData(odata): OData,
) -> ApiResult<JsonPage<serde_json::Value>> {
    // This example supports cursor pagination + $select projection.
    // We intentionally do NOT support $filter/$orderby here (beyond the `statuses` param).
    if odata.filter.is_some() {
        return Err(modkit::api::bad_request(
            "$filter is not supported for this endpoint",
        ));
    }
    if !odata.order.0.is_empty() {
        return Err(modkit::api::bad_request(
            "$orderby is not supported for this endpoint",
        ));
    }

    let page = svc
        .list_tenants(&sec, query.to_filter(), odata.clone())
        .await?
        .map_items(TenantDto::from);

    Ok(Json(page_to_projected_json(&page, odata.selected_fields())))
}

/// GET /tenant-resolver/v1/tenants/{id}/parents
#[tracing::instrument(skip_all, fields(tenant.id = %id))]
pub async fn get_parents(
    Extension(sec): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Path(id): Path<String>,
    Query(query): Query<GetParentsQuery>,
) -> ApiResult<Json<GetParentsResponseDto>> {
    let response = svc
        .get_parents(&sec, &id, query.to_filter(), query.to_access_options())
        .await?;

    Ok(Json(response.into()))
}

/// GET /tenant-resolver/v1/tenants/{id}/children
#[tracing::instrument(skip_all, fields(tenant.id = %id))]
pub async fn get_children(
    Extension(sec): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Path(id): Path<String>,
    Query(query): Query<GetChildrenQuery>,
) -> ApiResult<Json<GetChildrenResponseDto>> {
    let children = svc
        .get_children(
            &sec,
            &id,
            query.to_filter(),
            query.to_access_options(),
            query.max_depth.unwrap_or(0),
        )
        .await?;

    Ok(Json(GetChildrenResponseDto {
        children: children.into_iter().map(Into::into).collect(),
    }))
}
