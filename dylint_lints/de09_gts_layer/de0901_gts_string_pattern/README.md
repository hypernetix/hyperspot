# DE0901 – GTS string pattern validator

## What it does

`DE0901_GTS_STRING_PATTERN` validates every string literal that looks like a
Global Type Schema (GTS) identifier. It ensures that:

1. `schema_id = "..."` inside `#[struct_to_gts_schema]` attributes is a valid
   **type schema** (must end with `~`, no wildcards).
2. Arguments passed to `gts_make_instance_id("...")` are valid **instance
   segment identifiers** (single segment, no wildcards, no `:` or `~`).
3. Any other string literal that starts with `gts.` or appears inside a
   colon-separated permission string contains a valid schema/instance chain.

Wildcards (`*`) are only allowed in contexts where they are used as patterns:
permission strings, `resource_pattern(...)`, `with_pattern(...)`, and
`str.starts_with(...)`. Everywhere else, the lint rejects wildcard tokens so
that production code never ships with overly-broad or malformed identifiers.

Use `#[allow(de0901_gts_string_pattern)]` to suppress the lint:
```rust
#[allow(unknown_lints)]
#[allow(de0901_gts_string_pattern)]
let schema = "gts.acme.core.events.*";
```

## Why is this bad?

* Invalid identifiers break contract generation, registry lookups, or instance
  resolution at runtime.
* Wildcards in schema identifiers create ambiguous or insecure behavior, e.g.,
  allowing access to whole type families.
* Providing schemas to APIs that expect instance segments (or vice versa) leads
  to confusing errors buried deep inside infrastructure crates.

By catching the issues early, the lint prevents accidental schema typos and
protects security-critical permission checks.

## Known exceptions

* Permission strings (anything containing `:`) allow wildcards in the GTS
  segment, but the lint still validates each GTS component.
* `resource_pattern("...")` and `with_pattern("...")` calls also allow
  wildcards, since they represent pattern builders.
* Strings passed to `str.starts_with("gts.")` are ignored.
* Inline suppressions are supported through `#[allow(de0901_gts_string_pattern)]`
  on a binding or expression when a wildcard must be hard-coded outside of the
  recognized helper APIs.

## Example

```rust
// ❌ Triggers DE0901: wildcard inside a plain schema string
let schema = "gts.acme.core.events.*";

// ❌ Triggers DE0901: schema (with `~`) used in gts_make_instance_id
let _id = Product::gts_make_instance_id("vendor.package.sku.some.v1~");
```

Use instead:

```rust
// ✅ Explicit type schema
let schema = "gts.acme.core.events.type.v1~";

// ✅ Instance id segment
let _id = Product::gts_make_instance_id("vendor.package.sku.some.v1");

// ✅ Wildcard allowed inside permission/resource patterns
let pattern = Permission::builder()
    .resource_pattern("gts.acme.core.events.topic.v1~vendor.*")
    .action("publish")
    .build()
    .unwrap();

// ✅ Inline suppression when a literal wildcard is unavoidable
#[allow(unknown_lints)]
#[allow(de0901_gts_string_pattern)]
let filter = "gts.acme.*";
let query = ListQuery::new().with_pattern(filter);
```
