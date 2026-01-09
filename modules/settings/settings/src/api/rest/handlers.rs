use std::sync::Arc;

use axum::{extract::Extension, Json};
use modkit::api::prelude::*;
use modkit_security::SecurityContext;

use crate::domain::service::Service;

use super::dto::{PatchSettingsRequest, SettingsDto, UpdateSettingsRequest};

pub async fn get_settings(
    Extension(svc): Extension<Arc<Service>>,
) -> ApiResult<JsonBody<SettingsDto>> {
    let ctx = SecurityContext::root();
    let settings = svc.get_settings(&ctx).await?;
    Ok(Json(settings.into()))
}

pub async fn update_settings(
    Extension(svc): Extension<Arc<Service>>,
    Json(req): Json<UpdateSettingsRequest>,
) -> ApiResult<impl IntoResponse> {
    let ctx = SecurityContext::root();
    let settings = svc.update_settings(&ctx, req.theme, req.language).await?;
    let dto: SettingsDto = settings.into();
    Ok((StatusCode::OK, Json(dto)))
}

pub async fn patch_settings(
    Extension(svc): Extension<Arc<Service>>,
    Json(req): Json<PatchSettingsRequest>,
) -> ApiResult<JsonBody<SettingsDto>> {
    let ctx = SecurityContext::root();
    let settings = svc.patch_settings(&ctx, req.into()).await?;
    Ok(Json(settings.into()))
}
