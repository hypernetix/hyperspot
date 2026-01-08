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
    pub fn handler<F>(self, _handler: F) -> Self {
        self
    }
    pub fn register(self) -> Self {
        self
    }
}

fn dummy_handler() {}

pub fn define_endpoints() {
    // Using query_param for $filter - should use with_odata_filter
    OperationBuilder::get("/users-info/v1/users")
        // Should trigger DE0802 - use OData ext
        .query_param("$filter", false, "OData filter expression");

    // Using query_param for $orderby - should use with_odata_orderby
    OperationBuilder::get("/users-info/v1/users")
        // Should trigger DE0802 - use OData ext
        .query_param("$orderby", false, "OData ordering");

    // Using query_param for $select - should use with_odata_select
    OperationBuilder::get("/users-info/v1/users")
        // Should trigger DE0802 - use OData ext
        .query_param("$select", false, "OData field selection");

    // Using query_param_typed for $filter
    OperationBuilder::post("/users-info/v1/users")
        // Should trigger DE0802 - use OData ext
        .query_param_typed("$filter", false, "OData filter", "string");

    // Using query_param for $top
    OperationBuilder::get("/users-info/v1/users")
        // Should trigger DE0802 - use OData ext
        .query_param("$top", false, "Maximum number of results");

    // Using query_param for $skip
    OperationBuilder::get("/users-info/v1/users")
        // Should trigger DE0802 - use OData ext
        .query_param("$skip", false, "Number of results to skip");

    // Using query_param for $count
    OperationBuilder::get("/users-info/v1/users")
        // Should trigger DE0802 - use OData ext
        .query_param("$count", false, "Include total count");

    // Multiple OData params in chain
    OperationBuilder::get("/users-info/v1/users")
        // Should trigger DE0802 - use OData ext
        .query_param("$filter", false, "Filter")
        // Should trigger DE0802 - use OData ext
        .query_param("$orderby", false, "Order")
        .handler(dummy_handler)
        .register();
}
fn main() {}
