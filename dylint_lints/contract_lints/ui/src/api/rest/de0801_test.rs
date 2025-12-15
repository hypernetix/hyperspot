// Test DE0801: API Endpoint Must Have Version
pub struct OperationBuilder;

impl OperationBuilder {
    pub fn get(_path: &str) -> Self { Self }
    pub fn post(_path: &str) -> Self { Self }
    pub fn put(_path: &str) -> Self { Self }
}

pub fn define_endpoints() {
    // Should NOT trigger - has version
    OperationBuilder::get("/tests/v1/users");
    OperationBuilder::get("/abc/v2/products");
    OperationBuilder::post("/a-b-c/v1/orders");
    OperationBuilder::get("/tests/v1/users/{id}");
    OperationBuilder::post("/tests/v2/users/{id}/update");
    OperationBuilder::put("/tests/v3/products/{id}");

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

    // Should trigger DE0801 - no service name
    OperationBuilder::get("/v1/products");

    // Should trigger DE0801 - service name is not in kebab case
    OperationBuilder::get("/some_service/v1/products");

    // Should trigger DE0801 - capital letters
    OperationBuilder::get("/SomeService/v1/products");

    // Should trigger DE0801 - capital version
    OperationBuilder::get("/some-service/V1/products");

    // Should trigger DE0801 - capital endpoint
    OperationBuilder::get("/some-service/v1/Products");

    // Should trigger DE0801 - leading "-" in service name
    OperationBuilder::get("/-some-service/v1/products");

    // Should trigger DE0801 - leading "-" in resource name
    OperationBuilder::get("/some-service/v1/-products");

    // Should trigger DE0801 - leading "-" in sub-resource name
    OperationBuilder::get("/some-service/v1/products/-abc");
}
