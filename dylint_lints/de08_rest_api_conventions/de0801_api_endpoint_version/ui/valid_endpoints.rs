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
    
    // Simple GET with handler
    OperationBuilder::get("/tests/v1/users")
        .handler(list_users)
        .build();
    
    // POST with multiple methods
    OperationBuilder::post("/abc/v2/products")
        .handler(create_order);
    
    // Various HTTP methods
    OperationBuilder::post("/a-b-c/v1/orders");
    OperationBuilder::put("/tests/v1/users/{id}")
        .handler(update_product);
    OperationBuilder::delete("/tests/v2/users/{id}/profile");
    OperationBuilder::patch("/tests/v3/products/{id}");
    
    // Different service names and version numbers
    OperationBuilder::get("/my-service/v10/resources")
        .handler(get_user)
        .build();
    OperationBuilder::post("/service1/v1/items/{id}/details");
    
    // Path parameters in various positions
    OperationBuilder::get("/api-service/v5/users/{user-id}/orders/{order-id}")
        .handler(list_users);
}

fn main() {}
