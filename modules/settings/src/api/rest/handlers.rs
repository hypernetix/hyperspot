use crate::api::auth;
use crate::api::rest::dto::{SettingsDTO, UpdateSettingsReq};
use crate::api::rest::error::map_domain_error;
use crate::contract::model::SettingsPatch;
use crate::domain::service::Service;
use axum::http::Uri;
use axum::{Extension, Json};
use modkit::ProblemResponse;
use tracing::{error, info};

/// GET /settings endpoint
pub async fn get_settings(
    Extension(svc): Extension<std::sync::Arc<Service>>,
    uri: Uri,
) -> Result<Json<SettingsDTO>, ProblemResponse> {
    info!("Listing user's settings");

    match svc
        .get_settings(auth::get_user_id(), auth::get_tenant_id())
        .await
    {
        Ok(settings) => Ok(Json(SettingsDTO {
            user_id: settings.user_id,
            tenant_id: settings.tenant_id,
            theme: settings.theme,
            language: settings.language,
        })),

        Err(e) => {
            error!("Failed to get settings: {}", e);
            Err(map_domain_error(&e, uri.path()))
        }
    }
}

/// UPDATE /settings endpoint
pub async fn update_settings(
    Extension(svc): Extension<std::sync::Arc<Service>>,
    uri: Uri,
    Json(payload): Json<UpdateSettingsReq>,
) -> Result<Json<SettingsDTO>, ProblemResponse> {
    info!("Listing user's settings");

    match svc
        .update_settings(
            auth::get_user_id(),
            auth::get_tenant_id(),
            SettingsPatch::new(payload.theme, payload.language),
        )
        .await
    {
        Ok(settings) => Ok(Json(SettingsDTO {
            user_id: settings.user_id,
            tenant_id: settings.tenant_id,
            theme: settings.theme,
            language: settings.language,
        })),

        Err(e) => {
            error!("Failed to get settings: {}", e);
            Err(map_domain_error(&e, uri.path()))
        }
    }
}
