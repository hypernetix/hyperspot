# ModKit Errors Macro

Proc-macro for generating strongly-typed error catalogs from JSON.

## Overview

The `cf-modkit-errors-macro` crate provides the `declare_errors!` macro.

```rust,ignore
declare_errors! {
    path = "gts/errors_system.json",
    namespace = "system_errors",
    vis = "pub"
}
```

## License

Licensed under Apache-2.0.
