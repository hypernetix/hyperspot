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

fn dummy_handler() {}

pub fn define_endpoints() {
    // Missing service name and version
    OperationBuilder::get("/users");

    // Missing service name (looks like version but list is not valid version)
    OperationBuilder::get("/users/list")
        .handler(dummy_handler);

    // Second segment not a valid version
    OperationBuilder::post("/api/users")
        .handler(dummy_handler)
        .build();

    // Invalid version format
    OperationBuilder::put("/version1/users");

    // Missing service name (version first)
    OperationBuilder::delete("/v1/products")
        .handler(dummy_handler);

    // Service name with underscore (not kebab-case)
    OperationBuilder::patch("/some_service/v1/products");

    // Service name with capital letters
    OperationBuilder::get("/SomeService/v1/products")
        .handler(dummy_handler)
        .build();

    // Uppercase version
    OperationBuilder::post("/some-service/V1/products");

    // Capital letter in resource name
    OperationBuilder::put("/some-service/v1/Products")
        .handler(dummy_handler);

    // Leading dash in service name
    OperationBuilder::get("/-some-service/v1/products");

    // Leading dash in resource name
    OperationBuilder::delete("/some-service/v1/-products")
        .handler(dummy_handler);

    // Leading dash in sub-resource
    OperationBuilder::patch("/some-service/v1/products/-abc");

    // Missing resource after version
    OperationBuilder::post("/my-service/v1")
        .handler(dummy_handler)
        .build();
}

fn main() {}
