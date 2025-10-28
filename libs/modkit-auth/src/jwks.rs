use crate::{claims::Claims, errors::AuthError, traits::TokenValidator};
use async_trait::async_trait;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    #[serde(rename = "use")]
    #[allow(dead_code)]
    use_: Option<String>,
    n: String,
    e: String,
    #[allow(dead_code)]
    alg: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

/// JWKS-based JWT validator
pub struct JwksValidator {
    jwks_uri: String,
    expected_issuer: Option<String>,
    expected_audience: Option<String>,
    keys: Arc<RwLock<HashMap<String, DecodingKey>>>,
}

impl JwksValidator {
    pub fn new(
        jwks_uri: String,
        expected_issuer: Option<String>,
        expected_audience: Option<String>,
    ) -> Self {
        Self {
            jwks_uri,
            expected_issuer,
            expected_audience,
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Fetch JWKS from the endpoint and cache keys
    async fn fetch_keys(&self) -> Result<(), AuthError> {
        let response = reqwest::get(&self.jwks_uri)
            .await
            .map_err(|e| AuthError::JwksFetchFailed(e.to_string()))?;

        let jwks: JwksResponse = response
            .json()
            .await
            .map_err(|e| AuthError::JwksFetchFailed(e.to_string()))?;

        let mut keys = HashMap::new();
        for jwk in jwks.keys {
            if jwk.kty == "RSA" {
                let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
                    .map_err(|e| AuthError::JwksFetchFailed(e.to_string()))?;
                keys.insert(jwk.kid, key);
            }
        }

        *self.keys.write().await = keys;
        Ok(())
    }

    /// Get a decoding key by kid, refreshing if necessary
    async fn get_key(&self, kid: &str) -> Result<DecodingKey, AuthError> {
        // Try reading from cache first
        {
            let keys = self.keys.read().await;
            if let Some(key) = keys.get(kid) {
                return Ok(key.clone());
            }
        }

        // Cache miss - fetch keys and try again
        self.fetch_keys().await?;

        let keys = self.keys.read().await;
        keys.get(kid)
            .cloned()
            .ok_or_else(|| AuthError::InvalidToken(format!("Unknown key ID: {}", kid)))
    }
}

#[async_trait]
impl TokenValidator for JwksValidator {
    async fn validate_and_parse(&self, token: &str) -> Result<Claims, AuthError> {
        // Strip "Bearer " if present
        let token = token.trim_start_matches("Bearer ").trim();

        // Decode header and pick correct key
        let header = decode_header(token)
            .map_err(|e| AuthError::InvalidToken(format!("Invalid header: {}", e)))?;

        let kid = header
            .kid
            .clone()
            .ok_or_else(|| AuthError::InvalidToken("Missing kid".into()))?;

        let key = self.get_key(&kid).await?;

        // Prepare validation
        let mut validation = Validation::new(header.alg);

        // Keep expiration validation
        validation.validate_exp = true;

        // Optional: disable nbf unless needed
        validation.validate_nbf = false;

        // Disable built-in audience validation (causes InvalidAudience with Keycloak)
        validation.validate_aud = false;

        // Keep issuer validation if configured
        if let Some(iss) = &self.expected_issuer {
            validation.set_issuer(&[iss]);
        }

        // Decode raw claims into JSON (to handle flexible aud field)
        let data = decode::<Value>(token, &key, &validation)
            .map_err(|e| AuthError::InvalidToken(format!("JWT decode failed: {}", e)))?;

        let v = data.claims;

        // --- Normalize standard fields ---
        let sub = v
            .get("sub")
            .and_then(|x| x.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| AuthError::InvalidToken("Missing or invalid sub".into()))?;

        // aud can be string OR array (Keycloak compatibility)
        let aud = match v.get("aud") {
            Some(Value::String(s)) => Some(vec![s.clone()]),
            Some(Value::Array(arr)) => Some(
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        };

        // Optional fields
        let iss = v.get("iss").and_then(|x| x.as_str()).map(|s| s.to_string());
        let exp = v.get("exp").and_then(|x| x.as_i64());
        let iat = v.get("iat").and_then(|x| x.as_i64());
        let nbf = v.get("nbf").and_then(|x| x.as_i64());

        // --- Normalize custom claims ---
        // tenants: array of UUIDs
        let tenants = v
            .get("tenants")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str())
                    .filter_map(|s| Uuid::parse_str(s).ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // roles: top-level `roles` OR realm_access.roles (Keycloak compatibility)
        let mut roles: Vec<String> = Vec::new();
        if let Some(Value::Array(arr)) = v.get("roles") {
            roles.extend(arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())));
        } else if let Some(Value::Object(realm)) = v.get("realm_access") {
            if let Some(Value::Array(arr)) = realm.get("roles") {
                roles.extend(arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())));
            }
        }

        let email = v
            .get("email")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());

        // --- Construct Claims ---
        let claims = Claims {
            sub,
            iss,
            aud,
            exp,
            iat,
            nbf,
            tenants,
            roles,
            email,
        };

        // Quick validity checks
        if let Some(exp_ts) = claims.exp {
            let now = chrono::Utc::now().timestamp();
            if now >= exp_ts {
                return Err(AuthError::TokenExpired);
            }
        }

        if let Some(nbf_ts) = claims.nbf {
            let now = chrono::Utc::now().timestamp();
            if now < nbf_ts {
                return Err(AuthError::InvalidToken("Token not yet valid (nbf)".into()));
            }
        }

        // Validate issuer if expected (already done by jsonwebtoken, but double-check)
        if let Some(ref expected) = self.expected_issuer {
            match &claims.iss {
                Some(actual) if actual == expected => {}
                Some(actual) => {
                    return Err(AuthError::IssuerMismatch {
                        expected: expected.clone(),
                        actual: actual.clone(),
                    })
                }
                None => {
                    return Err(AuthError::InvalidToken(
                        "Missing issuer in token".to_string(),
                    ))
                }
            }
        }

        // --- Custom Audience Check (Keycloak-friendly) ---
        if let Some(expected) = &self.expected_audience {
            // 1️⃣ aud: string or array (already normalized into claims.aud)
            let aud_ok = claims
                .aud
                .as_ref()
                .map(|v| v.iter().any(|s| s == expected))
                .unwrap_or(false);

            // 2️⃣ azp: Keycloak client_id
            let azp_ok = v
                .get("azp")
                .and_then(|x| x.as_str())
                .map(|s| s == expected)
                .unwrap_or(false);

            // 3️⃣ resource_access: check if client_id exists as key
            let ra_ok = v
                .get("resource_access")
                .and_then(|x| x.as_object())
                .map(|obj| obj.contains_key(expected))
                .unwrap_or(false);

            if !(aud_ok || azp_ok || ra_ok) {
                return Err(crate::errors::AuthError::AudienceMismatch {
                    expected: vec![expected.clone()],
                    actual: claims.aud.clone().unwrap_or_default(),
                });
            }
        }

        Ok(claims)
    }
}

/// Simple in-memory validator for testing (NOT for production)
#[cfg(test)]
pub struct MockValidator;

#[cfg(test)]
#[async_trait]
impl TokenValidator for MockValidator {
    async fn validate_and_parse(&self, _token: &str) -> Result<Claims, AuthError> {
        Ok(Claims {
            sub: uuid::Uuid::new_v4(),
            iss: Some("mock".to_string()),
            aud: None,
            exp: None,
            iat: None,
            nbf: None,
            tenants: vec![],
            roles: vec![],
            email: None,
        })
    }
}
