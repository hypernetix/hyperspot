#![allow(clippy::use_debug)]
#![allow(clippy::expect_used)] // example code, not production

//! Configuration reference: form-based auth, all config fields, error handling.

use modkit_auth::{ClientAuthMethod, OAuthClientConfig, SecretString, Token, TokenError};
use modkit_http::HttpClientBuilder;

/// Form-based auth: some `IdPs` require credentials in the POST body
/// instead of a Basic header.
#[allow(dead_code)]
async fn form_auth() -> Result<(), Box<dyn std::error::Error>> {
    use modkit_auth::HttpClientBuilderExt;

    let token = Token::new(OAuthClientConfig {
        token_endpoint: Some("https://idp.example.com/oauth/token".parse()?),
        client_id: "my-service".into(),
        client_secret: SecretString::new("my-secret"),
        auth_method: ClientAuthMethod::Form,
        ..Default::default()
    })
    .await?;

    let client = HttpClientBuilder::new().with_bearer_auth(token).build()?;

    let _resp = client.get("https://api.example.com/data").send().await?;

    println!("Used Form auth (client_id/client_secret in POST body)");
    Ok(())
}

/// All config fields with their defaults.
fn config_overview() {
    println!("\n=== Configuration reference ===");

    let config = OAuthClientConfig {
        // Endpoint -- exactly one of these must be set:
        token_endpoint: Some(
            "https://idp.example.com/oauth/token"
                .parse()
                .expect("valid URL"),
        ),
        issuer_url: None, // mutually exclusive with token_endpoint

        // Credentials:
        client_id: "my-service".into(),
        client_secret: SecretString::new("my-secret"),
        scopes: vec!["api.read".into(), "api.write".into()],
        auth_method: ClientAuthMethod::Basic, // or ClientAuthMethod::Form

        // Vendor-specific headers (e.g. Azure requires a resource header):
        extra_headers: vec![("x-vendor-id".into(), "acme-corp".into())],

        // Refresh policy (defaults shown):
        refresh_offset: std::time::Duration::from_secs(30 * 60),
        jitter_max: std::time::Duration::from_secs(5 * 60),
        min_refresh_period: std::time::Duration::from_secs(10),
        default_ttl: std::time::Duration::from_secs(5 * 60),

        // HTTP client override (None = use defaults):
        http_config: None,
    };

    // Debug output redacts secrets:
    println!("  {config:?}");

    // Validate before use:
    match config.validate() {
        Ok(()) => println!("  Config is valid"),
        Err(e) => println!("  Config error: {e}"),
    }
}

/// Error handling patterns.
async fn error_handling() {
    println!("\n=== Error handling ===");

    let result = Token::new(OAuthClientConfig {
        token_endpoint: Some(
            "https://unreachable.example.com/token"
                .parse()
                .expect("valid URL"),
        ),
        client_id: "my-service".into(),
        client_secret: SecretString::new("my-secret"),
        ..Default::default()
    })
    .await;

    match result {
        Ok(_) => println!("  Token acquired"),
        Err(TokenError::Http(msg)) => println!("  HTTP error: {msg}"),
        Err(TokenError::InvalidResponse(msg)) => {
            println!("  Bad response: {msg}");
        }
        Err(TokenError::ConfigError(msg)) => {
            println!("  Config error: {msg}");
        }
        Err(e) => println!("  Other error: {e}"),
    }
}

#[tokio::main]
async fn main() {
    config_overview();
    error_handling().await;

    // Requires a real IDP -- uncomment to test:
    // form_auth().await.unwrap();
}
