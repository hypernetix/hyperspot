//! REST handlers for the Types Registry module.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query};
use axum::Json;
use modkit::api::prelude::*;
use types_registry_sdk::RegisterSummary;

use super::dto::{
    GtsEntityDto, ListEntitiesQuery, ListEntitiesResponse, RegisterEntitiesRequest,
    RegisterEntitiesResponse, RegisterResultDto, RegisterSummaryDto,
};
use crate::domain::error::DomainError;
use crate::domain::service::TypesRegistryService;

pub type TypesRegistryResult<T> = ApiResult<T, DomainError>;
pub type TypesRegistryApiError = ApiError<DomainError>;

/// POST /api/v1/types-registry/entities
///
/// Register GTS entities in batch.
/// REST API always validates entities, regardless of ready state.
/// However, REST API is blocked until service is ready.
pub async fn register_entities(
    Extension(service): Extension<Arc<TypesRegistryService>>,
    Json(req): Json<RegisterEntitiesRequest>,
) -> TypesRegistryResult<(StatusCode, Json<RegisterEntitiesResponse>)> {
    if !service.is_ready() {
        return Err(TypesRegistryApiError::from_domain(
            crate::domain::error::DomainError::NotInReadyMode,
        ));
    }

    let results = service.register_validated(req.entities);

    let summary = RegisterSummary::from_results(&results);
    let result_dtos: Vec<RegisterResultDto> = results.into_iter().map(Into::into).collect();

    let response = RegisterEntitiesResponse {
        summary: RegisterSummaryDto::from(summary),
        results: result_dtos,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// GET /api/v1/types-registry/entities
///
/// List GTS entities with optional filtering.
pub async fn list_entities(
    Extension(service): Extension<Arc<TypesRegistryService>>,
    Query(query): Query<ListEntitiesQuery>,
) -> TypesRegistryResult<Json<ListEntitiesResponse>> {
    if !service.is_ready() {
        return Err(TypesRegistryApiError::from_domain(
            crate::domain::error::DomainError::NotInReadyMode,
        ));
    }

    let list_query = query.to_list_query();

    let entities = service
        .list(&list_query)
        .map_err(TypesRegistryApiError::from_domain)?;

    let entity_dtos: Vec<GtsEntityDto> = entities.into_iter().map(Into::into).collect();
    let count = entity_dtos.len();

    Ok(Json(ListEntitiesResponse {
        entities: entity_dtos,
        count,
    }))
}

/// GET /api/v1/types-registry/entities/{gts_id}
///
/// Get a single GTS entity by its identifier.
pub async fn get_entity(
    Extension(service): Extension<Arc<TypesRegistryService>>,
    Path(gts_id): Path<String>,
) -> TypesRegistryResult<Json<GtsEntityDto>> {
    if !service.is_ready() {
        return Err(TypesRegistryApiError::from_domain(
            crate::domain::error::DomainError::NotInReadyMode,
        ));
    }

    let entity = service
        .get(&gts_id)
        .map_err(TypesRegistryApiError::from_domain)?;

    Ok(Json(entity.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::InMemoryGtsRepository;
    use gts::GtsConfig;
    use serde_json::json;

    fn default_config() -> GtsConfig {
        GtsConfig {
            entity_id_fields: vec!["$id".to_owned(), "gtsId".to_owned(), "id".to_owned()],
            schema_id_fields: vec!["$schema".to_owned(), "gtsTid".to_owned(), "type".to_owned()],
        }
    }

    fn create_service() -> Arc<TypesRegistryService> {
        let repo = Arc::new(InMemoryGtsRepository::new(default_config()));
        Arc::new(TypesRegistryService::new(repo))
    }

    #[tokio::test]
    async fn test_register_entities_returns_503_when_not_ready() {
        let service = create_service();
        // Service is not ready yet

        let req = RegisterEntitiesRequest {
            entities: vec![json!({
                "$id": "gts.acme.core.events.user_created.v1~",
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object"
            })],
        };

        let result = register_entities(Extension(service), Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_entities_returns_503_when_not_ready() {
        let service = create_service();
        // Service is not ready yet

        let query = ListEntitiesQuery::default();
        let result = list_entities(Extension(service), Query(query)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_entity_returns_503_when_not_ready() {
        let service = create_service();
        // Service is not ready yet

        let result = get_entity(
            Extension(service),
            Path("gts.acme.core.events.user_created.v1~".to_owned()),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_entities_handler_when_ready() {
        let service = create_service();
        service.switch_to_ready().unwrap();

        let req = RegisterEntitiesRequest {
            entities: vec![json!({
                "$id": "gts.acme.core.events.user_created.v1~",
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object"
            })],
        };

        let result = register_entities(Extension(service), Json(req)).await;
        assert!(result.is_ok());

        let (status, Json(response)) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response.summary.total, 1);
        assert_eq!(response.summary.succeeded, 1);
        assert_eq!(response.summary.failed, 0);
    }

    #[tokio::test]
    async fn test_list_entities_handler_when_ready() {
        let service = create_service();

        // Register entities via internal API (before ready)
        let _ = service.register(vec![
            json!({
                "$id": "gts.acme.core.events.user_created.v1~",
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object"
            }),
            json!({
                "$id": "gts.globex.core.events.order_placed.v1~",
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object"
            }),
        ]);
        service.switch_to_ready().unwrap();

        let query = ListEntitiesQuery::default();
        let result = list_entities(Extension(service), Query(query)).await;
        assert!(result.is_ok());

        let Json(response) = result.unwrap();
        assert_eq!(response.count, 2);
    }

    #[tokio::test]
    async fn test_get_entity_handler_when_ready() {
        let service = create_service();

        // Register entity via internal API (before ready)
        let _ = service.register(vec![json!({
            "$id": "gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        })]);
        service.switch_to_ready().unwrap();

        let result = get_entity(
            Extension(service),
            Path("gts.acme.core.events.user_created.v1~".to_owned()),
        )
        .await;
        assert!(result.is_ok());

        let Json(entity) = result.unwrap();
        assert_eq!(entity.gts_id, "gts.acme.core.events.user_created.v1~");
    }

    #[tokio::test]
    async fn test_get_entity_not_found() {
        let service = create_service();
        service.switch_to_ready().unwrap();

        let result = get_entity(
            Extension(service),
            Path("gts.unknown.pkg.ns.type.v1~".to_owned()),
        )
        .await;
        assert!(result.is_err());
    }
}
