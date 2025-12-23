#![allow(dead_code)]

pub struct OperationBuilder;

impl OperationBuilder {
    pub fn get(_path: &str) -> Self {
        Self
    }
    pub fn post(_path: &str) -> Self {
        Self
    }
    pub fn put(_path: &str) -> Self {
        Self
    }
    pub fn delete(_path: &str) -> Self {
        Self
    }
    pub fn patch(_path: &str) -> Self {
        Self
    }
    pub fn handler<F>(self, _handler: F) -> Self {
        self
    }
    pub fn build(self) -> Self {
        self
    }
}

fn list_users() {}
fn get_user() {}
fn create_order() {}
fn update_product() {}
fn delete_resource() {}

pub fn define_endpoints() {
    // Valid patterns: /{service-name}/v{N}/{resource}
    
    // Should not trigger DE0801 - API endpoint version
    // Simple GET with handler
    OperationBuilder::get("/tests/v1/users")
        .handler(list_users)
        .build();
    
    // Should not trigger DE0801 - API endpoint version
    // POST with multiple methods
    OperationBuilder::post("/abc/v2/products")
        .handler(create_order);
    
    // Should not trigger DE0801 - API endpoint version
    // Various HTTP methods
    OperationBuilder::post("/a-b-c/v1/orders");
    // Should not trigger DE0801 - API endpoint version
    OperationBuilder::put("/tests/v1/users/{id}")
        .handler(update_product);
    // Should not trigger DE0801 - API endpoint version
    OperationBuilder::delete("/tests/v2/users/{id}/profile");
    // Should not trigger DE0801 - API endpoint version
    OperationBuilder::patch("/tests/v3/products/{id}");
    
    // Should not trigger DE0801 - API endpoint version
    // Different service names and version numbers
    OperationBuilder::get("/my-service/v10/resources")
        .handler(get_user)
        .build();
    // Should not trigger DE0801 - API endpoint version
    OperationBuilder::post("/service1/v1/items/{id}/details");
    
    // Should not trigger DE0801 - API endpoint version
    // Path parameters in various positions
    OperationBuilder::get("/api-service/v5/users/{user-id}/orders/{order-id}")
        .handler(list_users);
}

fn main() {}
