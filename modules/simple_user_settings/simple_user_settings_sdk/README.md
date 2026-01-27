# Simple User Settings SDK

SDK crate for the simple user settings module.

## Overview

The `cf-simple-user-settings-sdk` crate provides:

- `SimpleUserSettingsClient` trait
- Model types (`SimpleUserSettings`, `SimpleUserSettingsPatch`, `SimpleUserSettingsUpdate`)
- Error type (`SettingsError`)

Consumers obtain the client from `ClientHub`.

```rust,ignore
use simple_user_settings_sdk::SimpleUserSettingsClient;

let client = hub.get::<dyn SimpleUserSettingsClient>()?;
let settings = client.get_settings(&ctx).await?;
```

## License

Licensed under Apache-2.0.
