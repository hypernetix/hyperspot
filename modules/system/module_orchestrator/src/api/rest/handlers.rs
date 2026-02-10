use axum::Extension;
use modkit::api::prelude::*;
use std::sync::Arc;

use super::dto::ModuleDto;
use crate::domain::service::ModulesService;

/// List all registered modules with their capabilities, instances, and deployment mode
pub async fn list_modules(
    Extension(svc): Extension<Arc<ModulesService>>,
) -> ApiResult<Json<Vec<ModuleDto>>> {
    let modules: Vec<ModuleDto> = svc.list_modules().iter().map(ModuleDto::from).collect();
    Ok(Json(modules))
}
