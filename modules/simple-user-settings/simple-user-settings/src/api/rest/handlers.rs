use std::sync::Arc;

use axum::{extract::Extension, Json};
use modkit::api::prelude::*;
use modkit_auth::axum_ext::Authz;
use simple_user_settings_sdk::models::SimpleUserSettingsUpdate;

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
    Extension(svc): Extension<Arc<Service>>,
    Json(req): Json<PatchSimpleUserSettingsRequest>,
) -> ApiResult<JsonBody<SimpleUserSettingsDto>> {
    let settings = svc.patch_settings(&ctx, req.into()).await?;
    Ok(Json(settings.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repo::SettingsRepository;
    use crate::domain::service::ServiceConfig;
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::{get, patch, post};
    use axum::Router;
    use modkit_security::SecurityContext;
    use serde_json::Value;
    use simple_user_settings_sdk::models::{SimpleUserSettings, SimpleUserSettingsPatch};
    use tower::ServiceExt as _;
    use uuid::Uuid;

    struct MockRepository {
        find_result: Option<SimpleUserSettings>,
        upsert_result: SimpleUserSettings,
    }

    #[async_trait]
    impl SettingsRepository for MockRepository {
        async fn find_by_user(
            &self,
            _ctx: &SecurityContext,
        ) -> anyhow::Result<Option<SimpleUserSettings>> {
            Ok(self.find_result.clone())
        }

        async fn upsert_full(
            &self,
            _ctx: &SecurityContext,
            _theme: Option<String>,
            _language: Option<String>,
        ) -> anyhow::Result<SimpleUserSettings> {
            Ok(self.upsert_result.clone())
        }

        async fn upsert_patch(
            &self,
            _ctx: &SecurityContext,
            _patch: SimpleUserSettingsPatch,
        ) -> anyhow::Result<SimpleUserSettings> {
            Ok(self.upsert_result.clone())
        }
    }

    fn create_test_service() -> Arc<Service> {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let repo = Arc::new(MockRepository {
            find_result: Some(SimpleUserSettings {
                user_id,
                tenant_id,
                theme: Some("dark".to_owned()),
                language: Some("en".to_owned()),
            }),
            upsert_result: SimpleUserSettings {
                user_id,
                tenant_id,
                theme: Some("dark".to_owned()),
                language: Some("en".to_owned()),
            },
        });
        Arc::new(Service::new(repo, ServiceConfig::default()))
    }

    fn create_test_router(service: Arc<Service>) -> Router {
        Router::new()
            .route("/get", get(get_settings))
            .route("/update", post(update_settings))
            .route("/patch", patch(patch_settings))
            .layer(Extension(service))
            .layer(Extension(SecurityContext::anonymous()))
    }

    #[tokio::test]
    async fn test_get_settings_handler_returns_json() {
        let service = create_test_service();
        let app = create_test_router(service);

        let request = Request::builder()
            .method("GET")
            .uri("/get")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["theme"], "dark");
        assert_eq!(json["language"], "en");
    }

    #[tokio::test]
    async fn test_update_settings_handler_returns_json() {
        let service = create_test_service();
        let app = create_test_router(service);

        let body = r#"{"theme":"light","language":"es"}"#;
        let request = Request::builder()
            .method("POST")
            .uri("/update")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["theme"], "dark");
        assert_eq!(json["language"], "en");
    }

    #[tokio::test]
    async fn test_patch_settings_handler_returns_json() {
        let service = create_test_service();
        let app = create_test_router(service);

        let body = r#"{"theme":"light"}"#;
        let request = Request::builder()
            .method("PATCH")
            .uri("/patch")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["theme"], "dark");
        assert_eq!(json["language"], "en");
    }
}
