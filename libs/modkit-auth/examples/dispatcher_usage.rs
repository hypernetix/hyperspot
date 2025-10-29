/// Example of using the new AuthDispatcher plugin system
///
/// This example demonstrates:
/// 1. Setting up an AuthDispatcher with Keycloak plugin (single mode)
/// 2. Configuring JWKS key provider
/// 3. Validating JWT tokens
/// 4. Using normalized claims
///
/// Run with: cargo run --example dispatcher_usage
use modkit_auth::{build_auth_dispatcher, AuthConfig, AuthModeConfig, JwksConfig, PluginConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Example 1: Setting up an AuthDispatcher with Keycloak (Single Mode)
    setup_keycloak_auth()?;

    // Example 2: Setting up with generic OIDC
    setup_oidc_auth()?;

    // Example 3: YAML configuration example
    show_yaml_config();

    Ok(())
}

/// Example: Setting up an AuthDispatcher with Keycloak
fn setup_keycloak_auth() -> Result<(), String> {
    println!("\n=== Example 1: Keycloak Configuration ===");

    let mut plugins = HashMap::new();
    plugins.insert(
        "keycloak".to_string(),
        PluginConfig::Keycloak {
            tenant_claim: "tenants".to_string(),
            client_roles: Some("modkit-api".to_string()),
            role_prefix: Some("kc_".to_string()),
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "keycloak".to_string(),
        },
        leeway_seconds: 60,
        issuers: vec!["https://keycloak.example.com/realms/my-realm".to_string()],
        audiences: vec!["modkit-api".to_string()],
        jwks: Some(JwksConfig {
            uri: "https://keycloak.example.com/realms/my-realm/protocol/openid-connect/certs"
                .to_string(),
            refresh_interval_seconds: 300,
            max_backoff_seconds: 3600,
        }),
        plugins,
    };

    let dispatcher = build_auth_dispatcher(&config).map_err(|e| e.to_string())?;

    println!("✅ Keycloak dispatcher created successfully");
    println!(
        "   Allowed issuers: {:?}",
        dispatcher.validation_config().allowed_issuers
    );
    println!(
        "   Allowed audiences: {:?}",
        dispatcher.validation_config().allowed_audiences
    );

    Ok(())
}

/// Example: Setting up with Generic OIDC
fn setup_oidc_auth() -> Result<(), String> {
    println!("\n=== Example 2: Generic OIDC Configuration ===");

    let mut plugins = HashMap::new();
    plugins.insert(
        "generic-oidc".to_string(),
        PluginConfig::Oidc {
            tenant_claim: "tenants".to_string(),
            roles_claim: "roles".to_string(),
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "generic-oidc".to_string(),
        },
        leeway_seconds: 120, // 2 minutes leeway
        issuers: vec!["https://auth.example.com".to_string()],
        audiences: vec!["my-api".to_string(), "my-app".to_string()],
        jwks: Some(JwksConfig {
            uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
            refresh_interval_seconds: 600, // 10 minutes
            max_backoff_seconds: 7200,     // 2 hours
        }),
        plugins,
    };

    let dispatcher = build_auth_dispatcher(&config).map_err(|e| e.to_string())?;

    println!("✅ OIDC dispatcher created successfully");
    println!(
        "   Leeway: {} seconds",
        dispatcher.validation_config().leeway_seconds
    );
    println!(
        "   Allowed audiences: {:?}",
        dispatcher.validation_config().allowed_audiences
    );

    Ok(())
}

/// Example: Show YAML configuration
fn show_yaml_config() {
    println!("\n=== Example 3: YAML Configuration ===");

    let yaml = r#"
# Keycloak configuration (single mode)
provider: "keycloak"
leeway_seconds: 60
issuers:
  - "https://keycloak.example.com/realms/my-realm"
audiences:
  - "modkit-api"
jwks:
  uri: "https://keycloak.example.com/realms/my-realm/protocol/openid-connect/certs"
  refresh_interval_seconds: 300
  max_backoff_seconds: 3600
plugins:
  keycloak:
    type: keycloak
    tenant_claim: "tenants"
    client_roles: "modkit-api"
    role_prefix: "kc_"
"#;

    println!("{}", yaml);

    // Try to parse it
    match serde_yaml::from_str::<AuthConfig>(yaml) {
        Ok(config) => {
            println!("✅ YAML parsed successfully");
            println!("   Provider: {}", config.mode.provider);
            println!("   Number of plugins: {}", config.plugins.len());
        }
        Err(e) => {
            println!("❌ YAML parsing failed: {}", e);
        }
    }

    println!("\n=== Generic OIDC Configuration ===");
    let yaml = r#"
provider: "generic-oidc"
leeway_seconds: 60
issuers:
  - "https://auth.example.com"
audiences:
  - "my-api"
jwks:
  uri: "https://auth.example.com/.well-known/jwks.json"
  refresh_interval_seconds: 300
  max_backoff_seconds: 3600
plugins:
  generic-oidc:
    type: oidc
    tenant_claim: "tenants"
    roles_claim: "roles"
"#;

    println!("{}", yaml);

    match serde_yaml::from_str::<AuthConfig>(yaml) {
        Ok(config) => {
            println!("✅ YAML parsed successfully");
            println!("   Provider: {}", config.mode.provider);
            println!("   Number of issuers: {}", config.issuers.len());
        }
        Err(e) => {
            println!("❌ YAML parsing failed: {}", e);
        }
    }
}
