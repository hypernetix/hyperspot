#![allow(dead_code)]

pub struct OperationBuilder;

impl OperationBuilder {
    pub fn get(_path: &str) -> Self {
        Self
    }
    pub fn post(_path: &str) -> Self {
        Self
    }
    pub fn query_param(self, _name: &str, _required: bool, _desc: &str) -> Self {
        self
    }
    pub fn query_param_typed(self, _name: &str, _required: bool, _desc: &str, _type: &str) -> Self {
        self
    }
    pub fn with_odata_filter<T>(self) -> Self {
        self
    }
    pub fn with_odata_orderby<T>(self) -> Self {
        self
    }
    pub fn with_odata_select(self) -> Self {
        self
    }
    pub fn handler<F>(self, _handler: F) -> Self {
        self
    }
    pub fn register(self) -> Self {
        self
    }
}

struct UserDtoFilterField;

fn dummy_handler() {}

pub fn define_endpoints() {
    // Should not trigger DE0802 - use OData ext (using proper OData extension methods)
    OperationBuilder::get("/users-info/v1/users")
        .with_odata_filter::<UserDtoFilterField>()
        .with_odata_orderby::<UserDtoFilterField>()
        .with_odata_select()
        .handler(dummy_handler)
        .register();

    // Should not trigger DE0802 - use OData ext (non-OData query params are fine)
    OperationBuilder::get("/users-info/v1/users")
        .query_param("limit", false, "Maximum number of results")
        .query_param("cursor", false, "Pagination cursor")
        .query_param("search", false, "Search term")
        .handler(dummy_handler)
        .register();

    // Should not trigger DE0802 - use OData ext (typed non-OData params are fine)
    OperationBuilder::post("/users-info/v1/users")
        .query_param_typed("limit", false, "Max results", "integer")
        .query_param_typed("offset", false, "Offset", "integer")
        .handler(dummy_handler)
        .register();

    // Should not trigger DE0802 - use OData ext (mixed valid usage)
    OperationBuilder::get("/users-info/v1/users")
        .with_odata_filter::<UserDtoFilterField>()
        .with_odata_select()
        .query_param("include_deleted", false, "Include soft-deleted records")
        .handler(dummy_handler)
        .register();
}

fn main() {}
