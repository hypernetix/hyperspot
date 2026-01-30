//! Domain models for license enforcement.
//!
//! These models are transport-agnostic (no serde) and represent the core
//! domain concepts of license enforcement.

use std::collections::HashSet;

/// License feature identifier.
///
/// Represents a global feature that requires license validation.
/// Uses `HyperSpot GTS` identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LicenseFeatureID {
    /// GTS ID of the license feature
    pub gts_id: String,
}

impl LicenseFeatureID {
    /// Create a new license feature ID.
    #[must_use]
    pub fn new(gts_id: String) -> Self {
        Self { gts_id }
    }

    /// Get the GTS ID as a string reference.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.gts_id
    }
}

impl From<String> for LicenseFeatureID {
    fn from(gts_id: String) -> Self {
        Self { gts_id }
    }
}

impl From<&str> for LicenseFeatureID {
    fn from(gts_id: &str) -> Self {
        Self {
            gts_id: gts_id.to_owned(),
        }
    }
}

/// Set of enabled global features for a tenant.
///
/// This represents the complete set of global features that are enabled
/// for a tenant's license.
pub type EnabledGlobalFeatures = HashSet<LicenseFeatureID>;

// ============================================================================
// Global Feature Constants
// ============================================================================

/// `HyperSpot GTS` identifiers for global license features.
///
/// These constants are provided for consumer convenience. The gateway does not
/// validate against this list - any GTS feature ID can be checked.
pub mod global_features {
    use super::LicenseFeatureID;

    /// Base platform feature - included in all licenses.
    pub const BASE: &str = "gts.x.core.lic.feat.v1~x.core.global.base.v1";

    /// Cyber Workspace chat feature.
    pub const CYBER_CHAT: &str = "gts.x.core.lic.feat.v1~x.core.global.cyber_chat.v1";

    /// Cyber Workspace employee agents feature.
    pub const CYBER_EMPLOYEE_AGENTS: &str =
        "gts.x.core.lic.feat.v1~x.core.global.cyber_employee_agents.v1";

    /// Cyber Workspace employee units feature.
    pub const CYBER_EMPLOYEE_UNITS: &str =
        "gts.x.core.lic.feat.v1~x.core.global.cyber_employee_units.v1";

    /// Helper to convert a constant to a `LicenseFeatureID`.
    #[must_use]
    pub fn to_feature_id(gts_id: &str) -> LicenseFeatureID {
        LicenseFeatureID::new(gts_id.to_owned())
    }
}
