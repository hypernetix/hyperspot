//! Shared token across multiple service clients.
//!
//! One `Token` provider is shared via `Clone` (`Arc` inside, cheap).
//! All clients reuse the same background refresh.
//!
//! Go equivalent:
//! ```go
//! tp, _ := tokenprovider.New(...)
//! resolverClient := tenantsresolver.NewClient(addr1, tp, ...)
//! registryClient := typesregistry.NewClient(addr2, tp, ...)
//! ```
//!
//! NOTE: Requires a running IDP. Meant as an API reference, not a runnable demo.

use modkit_auth::{HttpClientBuilderExt, OAuthClientConfig, SecretString, Token};
use modkit_http::HttpClientBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = Token::new(OAuthClientConfig {
        token_endpoint: Some("https://idp.example.com/oauth/token".parse()?),
        client_id: "my-service".into(),
        client_secret: SecretString::new("my-secret"),
        scopes: vec!["api".into()],
        ..Default::default()
    })
    .await?;

    // Multiple clients reuse the same token
    let resolver_client = HttpClientBuilder::new()
        .with_bearer_auth(token.clone()) // cheap Arc clone
        .build()?;

    let registry_client = HttpClientBuilder::new()
        .with_bearer_auth(token) // same token, same background refresh
        .build()?;

    let _r1 = resolver_client
        .get("https://resolver.example.com/tenants")
        .send()
        .await?;
    let _r2 = registry_client
        .get("https://registry.example.com/types")
        .send()
        .await?;

    println!("Both clients used the same token provider");
    Ok(())
}
