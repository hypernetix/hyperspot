use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use super::{info, no_content, ApiResult, Json, JsonBody, LanguageDto, SecurityContext, Service};

pub(super) async fn list_user_languages(
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    user_id: Uuid,
) -> ApiResult<JsonBody<Vec<LanguageDto>>> {
    info!(
        user_id = %user_id,
        requester_id = %ctx.subject_id(),
        "Listing user languages"
    );

    let languages = svc.list_user_languages(&ctx, user_id).await?;
    let dtos: Vec<LanguageDto> = languages.into_iter().map(LanguageDto::from).collect();

    Ok(Json(dtos))
}

pub(super) async fn assign_language_to_user(
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    user_id: Uuid,
    language_id: Uuid,
) -> ApiResult<Response> {
    info!(
        user_id = %user_id,
        language_id = %language_id,
        updater_id = %ctx.subject_id(),
        "Assigning language to user"
    );

    svc.assign_language_to_user(&ctx, user_id, language_id)
        .await?;

    Ok(no_content().into_response())
}

pub(super) async fn remove_language_from_user(
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    user_id: Uuid,
    language_id: Uuid,
) -> ApiResult<Response> {
    info!(
        user_id = %user_id,
        language_id = %language_id,
        deleter_id = %ctx.subject_id(),
        "Removing language from user"
    );

    svc.remove_language_from_user(&ctx, user_id, language_id)
        .await?;

    Ok(no_content().into_response())
}
