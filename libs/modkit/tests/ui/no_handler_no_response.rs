//! This test should fail to compile because register() is called without handler or response

use modkit::api::OperationBuilder;
use axum::Router;

struct DummyRegistry;
impl modkit::api::OpenApiRegistry for DummyRegistry {
    fn register_operation(&self, _: &modkit::api::OperationSpec) {}
    fn ensure_schema_raw(&self, root_name: &str, _: Vec<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>) -> String {
        root_name.to_string()
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

fn main() {
    let registry = DummyRegistry;
    let router = Router::new();

    // This should fail to compile - missing both handler and response
    let _ = OperationBuilder::<_, _, ()>::get("/tests/v1/test")
        .summary("Test endpoint")
        .description("A test endpoint")
        .register(router, &registry);
}
