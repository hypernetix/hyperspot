//! Domain models for license enforcement.
//!
//! These models are transport-agnostic (no serde) and represent the core
//! domain concepts of license enforcement.

use uuid::Uuid;

/// License feature identifier.
///
/// Represents a feature or capability that requires license validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LicenseFeature {
    /// GTS ID of the license feature
    pub gts_id: String,
}

impl LicenseFeature {
    /// Create a new license feature.
    #[must_use]
    pub fn new(gts_id: String) -> Self {
        Self { gts_id }
    }
}

/// License status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseStatus {
    /// License is valid and active
    Active,
    /// License has expired
    Expired,
    /// License is not found or not assigned
    NotFound,
    /// License is suspended
    Suspended,
}

/// Request to check license access for a feature.
#[derive(Debug, Clone)]
pub struct LicenseCheckRequest {
    /// Tenant ID to check license for
    pub tenant_id: Uuid,
    /// Feature to check access for
    pub feature: LicenseFeature,
}

/// Response from license check.
#[derive(Debug, Clone)]
pub struct LicenseCheckResponse {
    /// Whether access is granted
    pub allowed: bool,
    /// Status of the license
    pub status: LicenseStatus,
    /// Optional reason when access is denied
    pub reason: Option<String>,
}
