# DE0301: Duration fields in `*Config` must use humantime serde

This lint enforces a consistent serialization/deserialization format for `Duration` values inside configuration structs.

## What it does

- Flags any struct whose name ends with `Config`.
- Within those structs, checks each field typed as:
  - `std::time::Duration`
  - `Option<std::time::Duration>`
- Requires that those fields use the `modkit_utils::humantime_serde` serde adapter:
  - `Duration` fields must have:
    - `#[serde(with = "modkit_utils::humantime_serde")]`
  - `Option<Duration>` fields must have:
    - `#[serde(with = "modkit_utils::humantime_serde::option")]`

`default` is allowed but not required. `#[serde(default)]` may be separate from the `with` attribute.

## Why this is bad

Without an explicit `#[serde(with = "...")]` adapter, duration fields in configs often end up:

- Having inconsistent formats across modules.
- Being error-prone for humans editing YAML/JSON (e.g. milliseconds vs seconds ambiguity).

Using the shared `modkit_utils::humantime_serde` adapter standardizes config to human-friendly strings like `"30s"`.

## Examples

### Bad (fails DE0301)

Fully-qualified type:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub timeout: std::time::Duration,
}
```

Imported type:

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub timeout: Duration,
}
```

Optional duration without adapter:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub timeout: Option<std::time::Duration>,
}
```

### Good (passes DE0301)

`Duration`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(with = "modkit_utils::humantime_serde")]
    pub timeout: std::time::Duration,
}
```

`Option<Duration>`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(with = "modkit_utils::humantime_serde::option")]
    #[serde(default)]
    pub timeout: Option<std::time::Duration>,
}
```

## Notes / constraints

- The lint only targets structs whose name ends with `Config`.
- The lint checks type identity (not just textual matching) for `Duration` and `Option<Duration>`.
- The lint requires the field-level `#[serde(with = "...")]` attribute (struct-level serde config does not satisfy the rule).
