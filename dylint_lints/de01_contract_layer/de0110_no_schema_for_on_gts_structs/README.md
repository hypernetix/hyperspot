# DE0110: No `schema_for!` on GTS Structs

## What it does

Detects usage of `schemars::schema_for!()` macro on GTS-wrapped structs (those using `#[struct_to_gts_schema]`).

## Why is this bad?

GTS-wrapped structs **must** use `gts_json_schema_with_refs()` for schema generation because:

1. **Performance**: It is static (computed at compile time), so it's faster
2. **Correct `$id`**: It automatically sets the correct `$id` field, no need to do it manually
3. **Proper `$ref`s**: It generates proper schema with `$ref` references, while `schema_for!` inlines everything

## Example

```rust
// BAD - uses schemars::schema_for!() on a GTS struct
use schemars::schema_for;

#[struct_to_gts_schema(...)]
pub struct MyPluginSpec { ... }

let schema = schema_for!(MyPluginSpec);  // ❌ Will trigger DE0110
```

Use instead:

```rust
// GOOD - uses GTS-provided method
#[struct_to_gts_schema(...)]
pub struct MyPluginSpec { ... }

let schema = MyPluginSpec::gts_json_schema_with_refs();  // ✅ Correct
```

## Detection

The lint detects `schema_for!` macro invocations where the type argument has the `#[struct_to_gts_schema]` attribute. Types with this attribute implement the `gts::GtsSchema` trait and have a `GTS_SCHEMA_ID` constant.
