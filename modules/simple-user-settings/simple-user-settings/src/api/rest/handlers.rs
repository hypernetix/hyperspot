use std::sync::Arc;

use axum::{Json, extract::Extension};
use modkit::api::prelude::*;
use modkit_auth::axum_ext::Authz;
use simple_user_settings_sdk::models::SimpleUserSettingsUpdate;

use crate::api::rest::routes::ConcreteService;

use super::dto::{
    PatchSimpleUserSettingsRequest, SimpleUserSettingsDto, UpdateSimpleUserSettingsRequest,
};

pub async fn get_settings(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<ConcreteService>>,
) -> ApiResult<JsonBody<SimpleUserSettingsDto>> {
    let settings = svc.get_settings(&ctx).await?;
    Ok(Json(settings.into()))
}

pub async fn update_settings(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<ConcreteService>>,
    Json(req): Json<UpdateSimpleUserSettingsRequest>,
) -> ApiResult<impl IntoResponse> {
    let update = SimpleUserSettingsUpdate {
        theme: req.theme,
        language: req.language,
    };
    let settings = svc.update_settings(&ctx, update).await?;
    let dto: SimpleUserSettingsDto = settings.into();
    Ok((StatusCode::OK, Json(dto)))
}

pub async fn patch_settings(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<ConcreteService>>,
    Json(req): Json<PatchSimpleUserSettingsRequest>,
) -> ApiResult<JsonBody<SimpleUserSettingsDto>> {
    let settings = svc.patch_settings(&ctx, req.into()).await?;
    Ok(Json(settings.into()))
}
