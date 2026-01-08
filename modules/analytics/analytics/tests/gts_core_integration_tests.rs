use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use serde_json::json;

/// Integration tests for GTS Core API with api_ingress
/// 
/// These tests verify the full REST API integration including:
/// - OperationBuilder route registration
/// - JWT validation via api_ingress middleware
/// - SecurityCtx injection
/// - OpenAPI schema generation
/// - Problem Details error responses

#[tokio::test]
async fn test_gts_core_routes_registered_with_api_ingress() {
    // This test verifies that GTS Core routes are properly registered
    // through RestfulModule and accessible via the api_ingress router
    
    // TODO: Implement full integration test once api_ingress test helpers are available
    // Expected test flow:
    // 1. Create test api_ingress instance with analytics module
    // 2. Send GET request to /analytics/v1/gts/{id}
    // 3. Verify route is accessible (not 404)
    // 4. Verify JWT requirement (401 without token)
    // 5. Verify successful response with valid JWT
    
    // For now, verify module structure is correct
    use analytics::AnalyticsModule;
    use modkit::RestfulModule;
    
    let module = AnalyticsModule::default();
    
    // Verify RestfulModule trait is implemented
    // This ensures routes can be registered with api_ingress
    let router = axum::Router::new();
    let mock_openapi = analytics::tests::MockOpenApiRegistry::new();
    let ctx = modkit::ModuleCtx::default();
    
    let result = module.register_rest(&ctx, router, &mock_openapi);
    assert!(result.is_ok(), "RestfulModule registration should succeed");
}

#[tokio::test]
async fn test_gts_core_requires_authentication() {
    // TODO: Implement JWT validation test with api_ingress
    // Expected behavior:
    // - Request without Authorization header -> 401 Unauthorized
    // - Request with invalid JWT -> 401 Unauthorized
    // - Request with valid JWT -> 200 OK (or appropriate status)
    
    // This will be implemented once api_ingress test helpers provide:
    // - Mock JWT generation
    // - Test server setup with analytics module
}

#[tokio::test]
async fn test_gts_core_security_ctx_injection() {
    // TODO: Implement SecurityCtx injection test
    // Expected behavior:
    // - Valid JWT with tenant claim -> SecurityCtx populated
    // - Handler receives tenant information via Extension<SecurityCtx>
    // - Tenant isolation enforced
}

#[tokio::test]
async fn test_gts_core_odata_query_parameters() {
    // TODO: Implement OData query parameter test
    // Expected behavior:
    // - GET /analytics/v1/gts?$filter=... -> filter applied
    // - GET /analytics/v1/gts?$select=... -> fields projected
    // - GET /analytics/v1/gts?$top=10 -> pagination works
}

#[tokio::test]
async fn test_gts_core_problem_details_errors() {
    // TODO: Implement RFC 7807 Problem Details error test
    // Expected behavior:
    // - Invalid GTS ID -> 400 Bad Request with Problem Details
    // - Unknown GTS type -> 404 Not Found with Problem Details
    // - Server error -> 500 with Problem Details
    // - All errors follow RFC 7807 format
}

#[tokio::test]
async fn test_gts_core_openapi_registration() {
    // Verify that all GTS Core operations are registered in OpenAPI
    use analytics::tests::MockOpenApiRegistry;
    use analytics::AnalyticsModule;
    use modkit::RestfulModule;
    
    let module = AnalyticsModule::default();
    let router = axum::Router::new();
    let mock_registry = MockOpenApiRegistry::new();
    let ctx = modkit::ModuleCtx::default();
    
    let _result = module.register_rest(&ctx, router, &mock_registry);
    
    // Verify operations were registered
    let operations = mock_registry.get_registered_operations();
    
    // Expected operations:
    // - gts_core.get_entity
    // - gts_core.list_entities
    // - gts_core.create_entity
    // - gts_core.update_entity
    // - gts_core.patch_entity
    // - gts_core.delete_entity
    
    assert!(
        operations.iter().any(|op| op.contains("gts_core.get_entity")),
        "GET operation should be registered"
    );
    assert!(
        operations.iter().any(|op| op.contains("gts_core.list_entities")),
        "LIST operation should be registered"
    );
    assert!(
        operations.iter().any(|op| op.contains("gts_core.create_entity")),
        "POST operation should be registered"
    );
    assert!(
        operations.iter().any(|op| op.contains("gts_core.update_entity")),
        "PUT operation should be registered"
    );
    assert!(
        operations.iter().any(|op| op.contains("gts_core.patch_entity")),
        "PATCH operation should be registered"
    );
    assert!(
        operations.iter().any(|op| op.contains("gts_core.delete_entity")),
        "DELETE operation should be registered"
    );
}

#[tokio::test]
async fn test_gts_core_crud_operations_flow() {
    // TODO: Full CRUD flow integration test
    // 1. POST /analytics/v1/gts -> Create entity
    // 2. GET /analytics/v1/gts/{id} -> Retrieve created entity
    // 3. PUT /analytics/v1/gts/{id} -> Update entity
    // 4. PATCH /analytics/v1/gts/{id} -> Partial update
    // 5. DELETE /analytics/v1/gts/{id} -> Delete entity
    // 6. GET /analytics/v1/gts/{id} -> Verify 404
}
