# ModKit Utils

Small utility helpers used by CyberFabric / ModKit.

## Overview

The `cf-modkit-utils` crate currently provides optional serde support for `humantime`.

## Usage

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct Foo {
    #[serde(with = "modkit_utils::humantime_serde")]
    timeout: Duration,
}
```

## License

Licensed under Apache-2.0.
