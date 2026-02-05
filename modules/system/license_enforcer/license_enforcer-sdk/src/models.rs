//! Domain models for license enforcement.
//!
//! These models are transport-agnostic (no serde) and represent the core
//! domain concepts of license enforcement.

use std::collections::HashSet;

use gts::{GtsID, GtsInstanceId};
use gts_macros::struct_to_gts_schema;
use modkit::api::operation_builder::LicenseFeature;

use crate::LicenseEnforcerError;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.lic.feat.v1~",
    description = "Base schema for license feature",
    properties = "id,is_global,description"
)]
pub struct LicenseFeatureIdSpecV1 {
    id: GtsInstanceId,
    is_global: bool,
    description: &'static str,
}

impl LicenseFeatureIdSpecV1 {
    /// Convert a GTS ID to an instance ID.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The GTS ID has no type ID
    /// - The type ID doesn't match the expected schema ID
    /// - The GTS ID format is invalid (wrong number of segments)
    pub fn gts_id_to_instance_id(gts_id: &GtsID) -> Result<GtsInstanceId, LicenseEnforcerError> {
        let type_id =
            gts_id
                .get_type_id()
                .ok_or_else(|| LicenseEnforcerError::InvalidLicenseFeatureId {
                    message: format!("No GTS type ID found: '{}'", gts_id.as_ref()),
                    source: None,
                })?;
        if Self::gts_schema_id() != type_id.as_str() {
            return Err(LicenseEnforcerError::InvalidLicenseFeatureId {
                message: format!(
                    "Invalid type: expected '{}', got '{type_id}'",
                    Self::gts_schema_id()
                ),
                source: None,
            });
        }
        if gts_id.gts_id_segments.len() != 2 {
            return Err(LicenseEnforcerError::InvalidLicenseFeatureId {
                message: format!("Invalid LicenseFeatureID format: '{}'", gts_id.as_ref()),
                source: None,
            });
        }

        let segment = &gts_id.gts_id_segments[1].segment;
        let instance = Self::gts_make_instance_id(segment);
        Ok(instance)
    }
}

pub trait LicenseFeatureId: AsRef<str> + Send + Sync {
    fn to_gts(&self) -> GtsID;
    fn to_gts_instance(&self) -> GtsInstanceId;
}

impl LicenseFeature for &dyn LicenseFeatureId {}

impl From<&dyn LicenseFeatureId> for GtsID {
    fn from(value: &dyn LicenseFeatureId) -> Self {
        value.to_gts()
    }
}

impl From<&dyn LicenseFeatureId> for GtsInstanceId {
    fn from(value: &dyn LicenseFeatureId) -> Self {
        value.to_gts_instance()
    }
}

impl From<&dyn LicenseFeatureId> for String {
    fn from(value: &dyn LicenseFeatureId) -> Self {
        value.to_gts_instance().as_ref().to_owned()
    }
}

/// Create a license feature ID from a GTS ID.
///
/// # Errors
///
/// Returns an error if the GTS ID is not valid for a license feature.
pub fn license_feature_id_from_gts_id(
    gts_id: GtsID,
) -> Result<Box<dyn LicenseFeatureId>, LicenseEnforcerError> {
    struct SimpleFeatureId {
        gts_id: GtsID,
        instance_id: GtsInstanceId,
    }

    impl AsRef<str> for SimpleFeatureId {
        fn as_ref(&self) -> &str {
            self.gts_id.as_ref()
        }
    }

    impl LicenseFeatureId for SimpleFeatureId {
        fn to_gts(&self) -> GtsID {
            self.gts_id.clone()
        }

        fn to_gts_instance(&self) -> GtsInstanceId {
            self.instance_id.clone()
        }
    }
    let instance = LicenseFeatureIdSpecV1::gts_id_to_instance_id(&gts_id)?;

    Ok(Box::new(SimpleFeatureId {
        gts_id,
        instance_id: instance,
    }))
}

/// Parse a license feature ID from a string.
///
/// # Errors
///
/// Returns an error if the string is not a valid GTS ID for a license feature.
pub fn parse_license_feature_id(
    val: &str,
) -> Result<Box<dyn LicenseFeatureId>, LicenseEnforcerError> {
    let gts_id = GtsID::new(val).map_err(|e| LicenseEnforcerError::InvalidLicenseFeatureId {
        message: format!("Invalid GTS ID format: '{val}'"),
        source: Some(Box::new(e)),
    })?;
    license_feature_id_from_gts_id(gts_id)
}

/// Set of enabled global features for a tenant.
///
/// This represents the complete set of global features that are enabled
/// for a tenant's license.
pub type EnabledGlobalFeatures = HashSet<GtsID>;

macro_rules! global_feature {
    ($feature:ident, $segment:expr, $desc:expr) => {
        pub struct $feature;

        impl LicenseFeatureId for $feature {
            fn to_gts(&self) -> gts::GtsID {
                gts::GtsID::new(&format!(
                    "{}{}",
                    LicenseFeatureIdSpecV1::gts_schema_id(),
                    $segment
                ))
                .expect("Valid GTS ID")
            }

            fn to_gts_instance(&self) -> GtsInstanceId {
                LicenseFeatureIdSpecV1::gts_make_instance_id($segment)
            }
        }

        impl AsRef<str> for $feature {
            fn as_ref(&self) -> &str {
                concat!("gts.x.core.lic.feat.v1~", $segment)
            }
        }

        impl From<&$feature> for LicenseFeatureIdSpecV1 {
            fn from(value: &$feature) -> Self {
                LicenseFeatureIdSpecV1 {
                    id: value.to_gts_instance(),
                    is_global: true,
                    description: $desc,
                }
            }
        }
    };
}

pub mod global_features {

    use gts::GtsInstanceId;

    use crate::models::{LicenseFeatureId, LicenseFeatureIdSpecV1};

    global_feature!(
        BaseFeature,
        "x.core.global.base.v1",
        "Base feature - included in all licenses."
    );
    global_feature!(
        CyberChatFeature,
        "x.core.global.cyber_chat.v1",
        "Cyber Workspace chat feature."
    );
    global_feature!(
        CyberEmployeeAgentsFeature,
        "x.core.global.cyber_employee_agents.v1",
        "Cyber Workspace employee agents feature."
    );
    global_feature!(
        CyberEmployeeUnitsFeature,
        "x.core.global.cyber_employee_units.v1",
        "Cyber Workspace employee units feature."
    );
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn test_parse_license_feature_id_happy_path() {
        let valid_id = "gts.x.core.lic.feat.v1~x.core.global.base.v1";
        let result = parse_license_feature_id(valid_id);
        assert!(result.is_ok());
        let feature_id = result.unwrap();
        assert_eq!(AsRef::<str>::as_ref(&*feature_id), valid_id);
        assert_eq!(feature_id.to_gts().as_ref(), valid_id);
    }

    #[test]
    fn test_parse_license_feature_id_invalid_gts_format() {
        let invalid_id = "invalid";
        let result = parse_license_feature_id(invalid_id);
        assert!(matches!(
            result,
            Err(LicenseEnforcerError::InvalidLicenseFeatureId { .. })
        ));
    }

    #[allow(unknown_lints)]
    #[allow(de0901_gts_string_pattern)]
    #[test]
    fn test_parse_license_feature_id_no_type_id() {
        let invalid_id = "gts.x.core.lic.feat.v1"; // missing ~
        let result = parse_license_feature_id(invalid_id);
        assert!(matches!(
            result,
            Err(LicenseEnforcerError::InvalidLicenseFeatureId { .. })
        ));
    }

    #[allow(unknown_lints)]
    #[allow(de0901_gts_string_pattern)]
    #[test]
    fn test_parse_license_feature_id_wrong_type_id() {
        let invalid_id = "gts.x.other.v1~x.core.global.base.v1";
        let result = parse_license_feature_id(invalid_id);
        assert!(matches!(
            result,
            Err(LicenseEnforcerError::InvalidLicenseFeatureId { .. })
        ));
    }

    #[allow(unknown_lints)]
    #[allow(de0901_gts_string_pattern)]
    #[test]
    fn test_parse_license_feature_id_wrong_segment_count() {
        let invalid_id = "gts.x.core.lic.feat.v1~segment1~segment2"; // 3 segments
        let result = parse_license_feature_id(invalid_id);
        assert!(matches!(
            result,
            Err(LicenseEnforcerError::InvalidLicenseFeatureId { .. })
        ));
    }

    #[test]
    fn test_base_feature_to_gts_returns_full_gts_id() {
        use crate::models::global_features::BaseFeature;

        let feature = BaseFeature;
        let gts_id = feature.to_gts();

        // to_gts() returns a GtsID, and as_ref() returns the full GTS ID string
        assert_eq!(
            gts_id.as_ref(),
            "gts.x.core.lic.feat.v1~x.core.global.base.v1",
            "to_gts().as_ref() should return the full GTS ID including type and instance"
        );
    }

    #[test]
    fn test_base_feature_to_gts_instance_returns_full_gts_id() {
        use crate::models::global_features::BaseFeature;

        let feature = BaseFeature;
        let instance_id = feature.to_gts_instance();

        // CURRENT BEHAVIOR: to_gts_instance() returns a GtsInstanceId,
        // but as_ref() returns the FULL GTS ID string (same as to_gts().as_ref())
        assert_eq!(
            instance_id.as_ref(),
            "gts.x.core.lic.feat.v1~x.core.global.base.v1",
            "to_gts_instance().as_ref() currently returns the full GTS ID (not just instance portion)"
        );
    }

    #[test]
    fn test_base_feature_as_ref_returns_full_gts_id() {
        use crate::models::global_features::BaseFeature;

        let feature = BaseFeature;

        // AsRef<str> implementation should return the full GTS ID
        assert_eq!(
            AsRef::<str>::as_ref(&feature),
            "gts.x.core.lic.feat.v1~x.core.global.base.v1",
            "AsRef<str> should return the full GTS ID"
        );
    }
}
