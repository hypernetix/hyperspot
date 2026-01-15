# de0803_api_snake_case

### What it does
Checks that DTOs (structs and enums) defined in the `api/rest` directory use `snake_case` for serde renaming configurations. Specifically:
1. `#[serde(rename_all = "...")]` on structs/enums must be "snake_case".
2. `#[serde(rename = "...")]` on fields must be in snake_case.

### Why is this bad?
The API standard requires all JSON properties to be in `snake_case`. Using other casing styles (like `camelCase`, `PascalCase`, etc.) leads to inconsistent API responses and violations of the project's API guidelines.

### Example

```rust
// Bad: using camelCase for rename_all
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyDto {
    pub my_field: String,
}

// Bad: using camelCase for field rename
#[derive(Serialize, Deserialize)]
pub struct AnotherDto {
    #[serde(rename = "myField")]
    pub my_field: String,
}
```

Use instead:

```rust
// Good: using snake_case for rename_all (or omitting it if fields are already snake_case)
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MyDto {
    pub my_field: String,
}

// Good: using snake_case for field rename
#[derive(Serialize, Deserialize)]
pub struct AnotherDto {
    #[serde(rename = "my_field")]
    pub my_field: String,
}
```
