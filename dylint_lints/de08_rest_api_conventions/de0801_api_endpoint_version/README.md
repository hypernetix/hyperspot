# DE0801: API Endpoint Must Have Service Name and Version

### What it does

Checks that all API endpoints follow the format `/{service-name}/v{N}/{resource}` where:
- `{service-name}` is in kebab-case (lowercase letters, numbers, dashes)
- `v{N}` is a version number (v1, v2, v10, etc.)
- `{resource}` is the resource path in kebab-case

### Why is this bad?

Consistent API endpoint structure is essential for:
- **Service identification**: Clearly identify which microservice owns an endpoint
- **API versioning**: Support multiple API versions simultaneously
- **Discoverability**: Predictable URL patterns for API consumers
- **Routing**: Easier to implement API gateways and load balancers
- **Documentation**: Clear organization in API documentation

Without this structure:
- Unclear which service owns which endpoints
- Difficult to version APIs without breaking changes
- Inconsistent API design across services
- Poor developer experience

### Validation Rules

1. **Service name** (first segment):
   - Must be kebab-case (lowercase letters, numbers, dashes)
   - Cannot start or end with a dash
   - Examples: `user-service`, `api-v2`, `product-catalog`

2. **Version** (second segment):
   - Must be `v` followed by digits only
   - Examples: `v1`, `v2`, `v10`
   - Not allowed: `V1`, `version1`, `v1.0`

3. **Resource** (third segment onwards):
   - Must be kebab-case
   - Path parameters like `{id}` are allowed
   - Examples: `users`, `user-profiles`, `orders/{order-id}`

### Example

```rust
// ❌ Bad - various violations
use modkit::api::OperationBuilder;

// Missing service name and version
OperationBuilder::get("/users");

// Missing service name (version first)
OperationBuilder::get("/v1/products");

// Service name not kebab-case (has underscore)
OperationBuilder::post("/some_service/v1/products");

// Uppercase letters in service name
OperationBuilder::get("/SomeService/v1/users");

// Uppercase version
OperationBuilder::get("/my-service/V1/products");

// Resource name not kebab-case
OperationBuilder::get("/my-service/v1/Products");
```

Use instead:

```rust
// ✅ Good - correct format
use modkit::api::OperationBuilder;

// Basic endpoint
OperationBuilder::get("/my-service/v1/users")
    .handler(list_users)
    .build();

// With path parameters
OperationBuilder::get("/my-service/v1/users/{id}")
    .handler(get_user);

// With sub-resources
OperationBuilder::post("/user-service/v2/users/{id}/profile")
    .handler(update_profile);

// Different versions coexist
OperationBuilder::get("/api-gateway/v1/health");
OperationBuilder::get("/api-gateway/v2/health");
```

### Configuration

This lint is configured to **deny** by default.

It checks all calls to `OperationBuilder` HTTP methods (get, post, put, delete, patch).

### See Also

- REST API best practices
- Semantic versioning
- API gateway routing patterns
