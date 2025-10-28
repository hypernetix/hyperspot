use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Standard JWT claims plus our custom claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,

    /// Issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,

    /// Audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<Vec<String>>,

    /// Expiration time (unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,

    /// Issued at time (unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,

    /// Not before time (unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,

    /// Custom: tenant IDs the user has access to
    #[serde(default)]
    pub tenants: Vec<Uuid>,

    /// Custom: roles assigned to the user
    #[serde(default)]
    pub roles: Vec<String>,

    /// Custom: email (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

impl Claims {
    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.exp {
            let now = chrono::Utc::now().timestamp();
            now >= exp
        } else {
            false
        }
    }

    /// Check if the token is valid yet (nbf check)
    pub fn is_valid_yet(&self) -> bool {
        if let Some(nbf) = self.nbf {
            let now = chrono::Utc::now().timestamp();
            now >= nbf
        } else {
            true
        }
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}
