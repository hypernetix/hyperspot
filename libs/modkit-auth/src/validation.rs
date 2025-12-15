use crate::{claims::Claims, claims_error::ClaimsError};
use time::OffsetDateTime;
use uuid::Uuid;

/// Configuration for common validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Allowed issuers (if empty, any issuer is accepted)
    pub allowed_issuers: Vec<String>,

    /// Allowed audiences (if empty, any audience is accepted)
    pub allowed_audiences: Vec<String>,

    /// Leeway in seconds for time-based validations (exp, nbf)
    pub leeway_seconds: i64,

    /// Require subject to be a valid UUID
    pub require_uuid_subject: bool,

    /// Require tenants to be valid UUIDs
    pub require_uuid_tenants: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            allowed_issuers: vec![],
            allowed_audiences: vec![],
            leeway_seconds: 60,
            require_uuid_subject: true,
            require_uuid_tenants: true,
        }
    }
}

/// Perform common validation checks on claims.
///
/// # Errors
/// Returns `ClaimsError` if any validation check fails (issuer, audience, expiration, etc.).
pub fn validate_claims(claims: &Claims, config: &ValidationConfig) -> Result<(), ClaimsError> {
    // 1. Validate issuer
    if !config.allowed_issuers.is_empty() && !config.allowed_issuers.contains(&claims.issuer) {
        return Err(ClaimsError::InvalidIssuer {
            expected: config.allowed_issuers.clone(),
            actual: claims.issuer.clone(),
        });
    }

    // 2. Validate audience (at least one must match)
    if !config.allowed_audiences.is_empty() {
        let has_valid_audience = claims
            .audiences
            .iter()
            .any(|aud| config.allowed_audiences.contains(aud));

        if !has_valid_audience {
            return Err(ClaimsError::InvalidAudience {
                expected: config.allowed_audiences.clone(),
                actual: claims.audiences.clone(),
            });
        }
    }

    // 3. Validate expiration with leeway
    if let Some(exp) = claims.expires_at {
        let now = OffsetDateTime::now_utc();
        let leeway = time::Duration::seconds(config.leeway_seconds);

        if now > exp + leeway {
            return Err(ClaimsError::Expired);
        }
    }

    // 4. Validate not-before with leeway
    if let Some(nbf) = claims.not_before {
        let now = OffsetDateTime::now_utc();
        let leeway = time::Duration::seconds(config.leeway_seconds);

        if now < nbf - leeway {
            return Err(ClaimsError::NotYetValid);
        }
    }

    // 5. Validate subject is UUID (already validated during normalization, but double-check)
    if config.require_uuid_subject && claims.sub.is_nil() {
        // sub is already a Uuid type, so this is guaranteed
        // Just a safety check for future-proofing
        return Err(ClaimsError::InvalidClaimFormat {
            field: "sub".to_string(),
            reason: "subject cannot be nil UUID".to_string(),
        });
    }

    // 6. Validate tenants are UUIDs (already validated during normalization)
    if config.require_uuid_tenants {
        for tenant in &claims.tenants {
            if tenant.is_nil() {
                return Err(ClaimsError::InvalidClaimFormat {
                    field: "tenants".to_string(),
                    reason: "tenant ID cannot be nil UUID".to_string(),
                });
            }
        }
    }

    Ok(())
}

/// Helper to parse a UUID from a JSON value.
///
/// # Errors
/// Returns `ClaimsError::InvalidClaimFormat` if the value is not a valid UUID string.
pub fn parse_uuid_from_value(
    value: &serde_json::Value,
    field_name: &str,
) -> Result<Uuid, ClaimsError> {
    value
        .as_str()
        .ok_or_else(|| ClaimsError::InvalidClaimFormat {
            field: field_name.to_string(),
            reason: "must be a string".to_string(),
        })
        .and_then(|s| {
            Uuid::parse_str(s).map_err(|_| ClaimsError::InvalidClaimFormat {
                field: field_name.to_string(),
                reason: "must be a valid UUID".to_string(),
            })
        })
}

/// Helper to parse an array of UUIDs from a JSON value.
///
/// # Errors
/// Returns `ClaimsError::InvalidClaimFormat` if the value is not an array of valid UUID strings.
pub fn parse_uuid_array_from_value(
    value: &serde_json::Value,
    field_name: &str,
) -> Result<Vec<Uuid>, ClaimsError> {
    value
        .as_array()
        .ok_or_else(|| ClaimsError::InvalidClaimFormat {
            field: field_name.to_string(),
            reason: "must be an array".to_string(),
        })?
        .iter()
        .map(|v| parse_uuid_from_value(v, field_name))
        .collect()
}

/// Helper to parse timestamp (seconds since epoch) into `OffsetDateTime`.
///
/// # Errors
/// Returns `ClaimsError::InvalidClaimFormat` if the value is not a valid unix timestamp.
pub fn parse_timestamp(
    value: &serde_json::Value,
    field_name: &str,
) -> Result<OffsetDateTime, ClaimsError> {
    let ts = value
        .as_i64()
        .ok_or_else(|| ClaimsError::InvalidClaimFormat {
            field: field_name.to_string(),
            reason: "must be a number (unix timestamp)".to_string(),
        })?;

    OffsetDateTime::from_unix_timestamp(ts).map_err(|_| ClaimsError::InvalidClaimFormat {
        field: field_name.to_string(),
        reason: "invalid unix timestamp".to_string(),
    })
}

/// Helper to extract string from JSON value.
///
/// # Errors
/// Returns `ClaimsError::MissingClaim` if the value is not a string.
pub fn extract_string(value: &serde_json::Value, field_name: &str) -> Result<String, ClaimsError> {
    value
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| ClaimsError::MissingClaim(field_name.to_string()))
}

/// Helper to extract string array from JSON value (handles both string and array)
#[must_use]
pub fn extract_audiences(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::String(s) => vec![s.clone()],
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect(),
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_claims() -> Claims {
        Claims {
            sub: Uuid::new_v4(),
            issuer: "https://test.example.com".to_string(),
            audiences: vec!["api".to_string()],
            expires_at: Some(OffsetDateTime::now_utc() + time::Duration::hours(1)),
            not_before: None,
            tenants: vec![Uuid::new_v4()],
            roles: vec!["user".to_string()],
            extras: serde_json::Map::new(),
        }
    }

    #[test]
    fn test_valid_claims_pass() {
        let claims = create_test_claims();
        let config = ValidationConfig {
            allowed_issuers: vec!["https://test.example.com".to_string()],
            allowed_audiences: vec!["api".to_string()],
            ..Default::default()
        };

        assert!(validate_claims(&claims, &config).is_ok());
    }

    #[test]
    fn test_invalid_issuer_fails() {
        let claims = create_test_claims();
        let config = ValidationConfig {
            allowed_issuers: vec!["https://other.example.com".to_string()],
            allowed_audiences: vec![],
            ..Default::default()
        };

        let result = validate_claims(&claims, &config);
        assert!(matches!(result, Err(ClaimsError::InvalidIssuer { .. })));
    }

    #[test]
    fn test_invalid_audience_fails() {
        let claims = create_test_claims();
        let config = ValidationConfig {
            allowed_issuers: vec![],
            allowed_audiences: vec!["other-api".to_string()],
            ..Default::default()
        };

        let result = validate_claims(&claims, &config);
        assert!(matches!(result, Err(ClaimsError::InvalidAudience { .. })));
    }

    #[test]
    fn test_expired_token_fails() {
        let mut claims = create_test_claims();
        claims.expires_at = Some(OffsetDateTime::now_utc() - time::Duration::hours(1));

        let config = ValidationConfig::default();
        let result = validate_claims(&claims, &config);
        assert!(matches!(result, Err(ClaimsError::Expired)));
    }

    #[test]
    fn test_not_yet_valid_fails() {
        let mut claims = create_test_claims();
        claims.not_before = Some(OffsetDateTime::now_utc() + time::Duration::hours(1));

        let config = ValidationConfig::default();
        let result = validate_claims(&claims, &config);
        assert!(matches!(result, Err(ClaimsError::NotYetValid)));
    }

    #[test]
    fn test_leeway_allows_expired() {
        let mut claims = create_test_claims();
        claims.expires_at = Some(OffsetDateTime::now_utc() - time::Duration::seconds(30));

        let config = ValidationConfig {
            leeway_seconds: 60,
            ..Default::default()
        };

        assert!(validate_claims(&claims, &config).is_ok());
    }

    #[test]
    fn test_parse_uuid_from_value() {
        let uuid = Uuid::new_v4();
        let value = json!(uuid.to_string());

        let result = parse_uuid_from_value(&value, "test");
        assert_eq!(result.unwrap(), uuid);
    }

    #[test]
    fn test_parse_uuid_from_value_invalid() {
        let value = json!("not-a-uuid");
        let result = parse_uuid_from_value(&value, "test");
        assert!(matches!(
            result,
            Err(ClaimsError::InvalidClaimFormat { .. })
        ));
    }

    #[test]
    fn test_extract_audiences_string() {
        let value = json!("api");
        let audiences = extract_audiences(&value);
        assert_eq!(audiences, vec!["api"]);
    }

    #[test]
    fn test_extract_audiences_array() {
        let value = json!(["api", "ui"]);
        let audiences = extract_audiences(&value);
        assert_eq!(audiences, vec!["api", "ui"]);
    }
}
