# Security Guideline

## Input Validation

```rust
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(email)]
    pub email: String,

    #[validate(custom = "validate_password")]
    pub password: String,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("Password too short"));
    }
    Ok(())
}
```

## Secrets Management

- **Never commit secrets** to version control
- **Use environment variables** for configuration
- **Rotate secrets regularly**
- **Use secure random generation** for tokens

```rust
// Bad: hardcoded secret
const API_KEY: &str = "sk-1234567890abcdef";

// Good: environment variable
let api_key = std::env::var("API_KEY")
    .context("API_KEY environment variable not set")?;
```
