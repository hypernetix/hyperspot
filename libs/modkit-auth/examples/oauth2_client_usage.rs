#![allow(clippy::use_debug)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::expect_used)] // example code, not production
#![allow(dead_code)] // IDP-dependent examples are commented out in main()

/// Outbound `OAuth2` client credentials -- usage examples.
///
/// This is the Rust equivalent of the Go pattern:
///
/// ```go
/// tokenProvider, _ := tokenprovider.New(idpURL, clientID, clientSecret, ...)
/// resolverClient, _ := tenantsresolver.NewClient(address, tokenProvider, ...)
/// ```
///
/// NOTE: This example requires a running IDP and target API to work.
///       It is meant as a reference for how the API is used, not as a
///       runnable demo.
///
/// Run with: `cargo run --example oauth2_client_usage`
use modkit_auth::{
    ClientAuthMethod, HttpClientBuilderExt, OAuthClientConfig, SecretString, Token, TokenError,
};
use modkit_http::HttpClientBuilder;

// ---------------------------------------------------------------------------
// Example 1: Direct token endpoint (simplest case)
//
// Go equivalent:
//   tp, _ := tokenprovider.New(tokenURL, clientID, clientSecret, ...)
//   client := &http.Client{Transport: authTransport(tp)}
// ---------------------------------------------------------------------------
async fn direct_token_endpoint() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 1: Direct token endpoint ===");

    // Step 1 -- create token provider (like tokenprovider.New in Go)
    let token = Token::new(OAuthClientConfig {
        token_endpoint: Some("https://idp.example.com/oauth/token".parse()?),
        client_id: "my-service".into(),
        client_secret: SecretString::new("my-secret"),
        scopes: vec!["tenants.read".into(), "tenants.write".into()],
        ..Default::default()
    })
    .await?;

    // Step 2 -- create HTTP client with automatic Bearer injection
    // (like tenantsresolver.NewClient(addr, tokenProvider, ...) in Go)
    let client = HttpClientBuilder::new().with_bearer_auth(token).build()?;

    // Step 3 -- every request gets Authorization: Bearer <token> automatically
    let _resp = client
        .get("https://resolver.example.com/api/v1/tenants")
        .send()
        .await?;

    println!("  Request sent with auto-injected Bearer token");
    Ok(())
}

// ---------------------------------------------------------------------------
// Example 2: OIDC discovery (issuer URL -> .well-known -> token_endpoint)
//
// Go equivalent:
//   tp, _ := tokenprovider.New(issuerURL, clientID, clientSecret, ...)
//   // internally calls fetchTokenURL -> /.well-known/openid-configuration
// ---------------------------------------------------------------------------
async fn oidc_discovery() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 2: OIDC discovery ===");

    // Only provide the issuer URL -- the token endpoint is resolved
    // automatically via {issuer_url}/.well-known/openid-configuration
    let token = Token::new(OAuthClientConfig {
        issuer_url: Some("https://idp.example.com".parse()?),
        client_id: "my-service".into(),
        client_secret: SecretString::new("my-secret"),
        ..Default::default()
    })
    .await?;

    let client = HttpClientBuilder::new().with_bearer_auth(token).build()?;

    let _resp = client.get("https://api.example.com/data").send().await?;

    println!("  Token endpoint discovered and token acquired");
    Ok(())
}

// ---------------------------------------------------------------------------
// Example 3: Shared token across multiple service clients
//
// Go equivalent:
//   tp, _ := tokenprovider.New(...)
//   resolverClient := tenantsresolver.NewClient(addr1, tp, ...)
//   registryClient := typesregistry.NewClient(addr2, tp, ...)
// ---------------------------------------------------------------------------
async fn shared_token() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 3: Shared token across clients ===");

    // One token provider -- shared via Clone (Arc inside, cheap)
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

    println!("  Both clients used the same token provider");
    Ok(())
}

// ---------------------------------------------------------------------------
// Example 4: Token invalidation on 401 (force refresh)
//
// Go equivalent:
//   resp := client.Do(req)
//   if resp.StatusCode == 401 { tp.Invalidate() }
// ---------------------------------------------------------------------------
async fn invalidation_on_401() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 4: Invalidation on 401 ===");

    let token = Token::new(OAuthClientConfig {
        token_endpoint: Some("https://idp.example.com/oauth/token".parse()?),
        client_id: "my-service".into(),
        client_secret: SecretString::new("my-secret"),
        ..Default::default()
    })
    .await?;

    let client = HttpClientBuilder::new()
        .with_bearer_auth(token.clone())
        .build()?;

    let resp = client
        .get("https://api.example.com/resource")
        .send()
        .await?;

    // send() returns Ok for ALL HTTP statuses (including 401).
    // Check the status directly on the response.
    if resp.status() == http::StatusCode::UNAUTHORIZED {
        println!("  Got 401 -- invalidating token...");
        token.invalidate().await;
        // Retry with a fresh token (the client re-reads automatically)
        let _retry = client
            .get("https://api.example.com/resource")
            .send()
            .await?;
        println!("  Retry succeeded with new token");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Example 5: Form-based auth (some IdPs require credentials in POST body)
// ---------------------------------------------------------------------------
async fn form_auth() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 5: Form-based authentication ===");

    let token = Token::new(OAuthClientConfig {
        token_endpoint: Some("https://idp.example.com/oauth/token".parse()?),
        client_id: "my-service".into(),
        client_secret: SecretString::new("my-secret"),
        auth_method: ClientAuthMethod::Form, // credentials in POST body, not Basic header
        ..Default::default()
    })
    .await?;

    let client = HttpClientBuilder::new().with_bearer_auth(token).build()?;

    let _resp = client.get("https://api.example.com/data").send().await?;

    println!("  Used Form auth (client_id/client_secret in POST body)");
    Ok(())
}

// ---------------------------------------------------------------------------
// Example 6: Config overview (all fields with defaults)
// ---------------------------------------------------------------------------
fn config_overview() {
    println!("\n=== Example 6: Configuration reference ===");

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
        refresh_offset: std::time::Duration::from_secs(30 * 60), // 30 min before expiry
        jitter_max: std::time::Duration::from_secs(5 * 60),      // up to 5 min random jitter
        min_refresh_period: std::time::Duration::from_secs(10),  // min 10s between attempts
        default_ttl: std::time::Duration::from_secs(5 * 60),     // fallback if no expires_in

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

// ---------------------------------------------------------------------------
// Example 7: Error handling
// ---------------------------------------------------------------------------
async fn error_handling() {
    println!("\n=== Example 7: Error handling ===");

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
        Err(TokenError::InvalidResponse(msg)) => println!("  Bad response: {msg}"),
        Err(TokenError::ConfigError(msg)) => println!("  Config error: {msg}"),
        Err(e) => println!("  Other error: {e}"),
    }
}

#[tokio::main]
async fn main() {
    println!("OAuth2 Client Credentials -- Usage Examples");
    println!("============================================");
    println!();
    println!("NOTE: Examples 1-5 require a running IDP and will fail without one.");
    println!("      They demonstrate the API shape, not a runnable demo.");

    // Config overview and error handling work without a real IDP
    config_overview();
    error_handling().await;

    // These require a real IDP -- uncomment to test:
    // direct_token_endpoint().await.unwrap();
    // oidc_discovery().await.unwrap();
    // shared_token().await.unwrap();
    // invalidation_on_401().await.unwrap();
    // form_auth().await.unwrap();
}
