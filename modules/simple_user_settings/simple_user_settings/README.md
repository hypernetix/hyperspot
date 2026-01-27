# Simple User Settings Module

Simple settings module.

## Overview

The `cf-simple-user-settings` crate implements the module runtime and storage.
The public API surface is defined in `cf-simple-user-settings-sdk` and is re-exported here.

## Configuration

```yaml
modules:
  simple-user-settings:
    config:
      max_field_length: 100
```

## License

Licensed under Apache-2.0.
