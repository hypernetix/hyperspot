//! OIDC discovery: issuer URL -> `.well-known` -> `token_endpoint`.
//!
//! Only provide the issuer URL -- the token endpoint is resolved
//! automatically via `{issuer_url}/.well-known/openid-configuration`.
//!
//! NOTE: Requires a running IDP. Meant as an API reference, not a runnable demo.

use modkit_auth::{HttpClientBuilderExt, OAuthClientConfig, SecretString, Token};
use modkit_http::HttpClientBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = Token::new(OAuthClientConfig {
        issuer_url: Some("https://idp.example.com".parse()?),
        client_id: "my-service".into(),
        client_secret: SecretString::new("my-secret"),
        ..Default::default()
    })
    .await?;

    let client = HttpClientBuilder::new().with_bearer_auth(token).build()?;

    let _resp = client.get("https://api.example.com/data").send().await?;

    println!("Token endpoint discovered and token acquired");
    Ok(())
}
