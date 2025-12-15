// Test DE0801: API Endpoint Must Have Version
pub struct OperationBuilder;

impl OperationBuilder {
    pub fn get(_path: &str) -> Self { Self }
    pub fn post(_path: &str) -> Self { Self }
    pub fn put(_path: &str) -> Self { Self }
}

pub fn define_endpoints() {
    // Should NOT trigger - has version
    OperationBuilder::get("/v1/users");
    OperationBuilder::get("/v2/products");
    OperationBuilder::post("/v1/orders");
    OperationBuilder::get("/v1/users/{id}");
    OperationBuilder::post("/v2/users/{id}/update");
    OperationBuilder::put("/v3/products/{id}");

    // Should trigger DE0801 - no version
    OperationBuilder::get("/users");

    // Should trigger DE0801 - no version with path param
    OperationBuilder::get("/users/{id}");

    // Should trigger DE0801 - no version in nested path
    OperationBuilder::post("/users/{id}/activate");

    // Should trigger DE0801 - looks like version but not /vN/
    OperationBuilder::get("/api/users");

    // Should trigger DE0801 - invalid version format
    OperationBuilder::get("/version1/users");

    // Should trigger DE0801 - no version
    OperationBuilder::get("/api/products");
}
