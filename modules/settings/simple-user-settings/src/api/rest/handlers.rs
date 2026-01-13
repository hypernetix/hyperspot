use std::sync::Arc;

use axum::{extract::Extension, Json};
use modkit::api::prelude::*;
use modkit_auth::axum_ext::Authz;

use crate::domain::service::Service;

use super::dto::{
    PatchSimpleUserSettingsRequest, SimpleUserSettingsDto, UpdateSimpleUserSettingsRequest,
};

pub async fn get_settings(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
) -> ApiResult<JsonBody<SimpleUserSettingsDto>> {
    let settings = svc.get_settings(&ctx).await?;
    Ok(Json(settings.into()))
}

pub async fn update_settings(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
    Json(req): Json<UpdateSimpleUserSettingsRequest>,
) -> ApiResult<impl IntoResponse> {
    let settings = svc.update_settings(&ctx, req.theme, req.language).await?;
    let dto: SimpleUserSettingsDto = settings.into();
    Ok((StatusCode::OK, Json(dto)))
}

pub async fn patch_settings(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
    Json(req): Json<PatchSimpleUserSettingsRequest>,
) -> ApiResult<JsonBody<SimpleUserSettingsDto>> {
    let settings = svc.patch_settings(&ctx, req.into()).await?;
    Ok(Json(settings.into()))
}
