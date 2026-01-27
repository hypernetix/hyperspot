//! GTS ID validation logic

use std::path::PathBuf;

use gts::GtsID;
use serde::Serialize;

/// Represents a single GTS validation error
#[derive(Debug, Clone, Serialize)]
pub struct GtsError {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub gts_id: String,
    pub error: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub context: String,
}

/// Aggregated validation results
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<GtsError>,
    pub files_scanned: usize,
}

impl ValidationResult {
    #[must_use]
    pub fn new(files_scanned: usize) -> Self {
        Self {
            errors: Vec::new(),
            files_scanned,
        }
    }

    pub fn add_errors(&mut self, errors: Vec<GtsError>) {
        self.errors.extend(errors);
    }

    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Contexts where wildcards are allowed (in documentation)
const WILDCARD_ALLOWED_CONTEXTS: &[&str] = &[
    "pattern",
    "filter",
    "query",
    "$filter",
    "starts_with",
    "with_pattern",
    "resource_pattern",
    "discovery",
    "match",
    "wildcard",
    "differs from",
    "get",
    "list",
    "todo",
    "p1 -",
    "p2 -",
    "p3 -",
];

/// Contexts that indicate "bad example" or intentionally invalid identifiers
const SKIP_VALIDATION_CONTEXTS: &[&str] = &[
    "invalid",
    "wrong",
    "bad",
    "reject",
    "error",
    "fail",
    "\u{274c}",
    "\u{2717}",
    "should not",
    "must not",
    "not allowed",
    "given**",
    "**given**",
];

/// Example vendors used in documentation that are tolerated during vendor validation.
/// These are placeholder/example vendors commonly used in docs and tutorials.
pub const EXAMPLE_VENDORS: &[&str] = &[
    "acme",     // Classic example company name
    "globex",   // Another example company name
    "example",  // Generic example
    "demo",     // Demo purposes
    "test",     // Test purposes
    "sample",   // Sample code
    "tutorial", // Tutorial examples
];

/// Check if a vendor is an example/placeholder vendor that should be tolerated
#[must_use]
pub fn is_example_vendor(vendor: &str) -> bool {
    EXAMPLE_VENDORS.contains(&vendor)
}

/// Check if the GTS identifier is in a context where wildcards are allowed
#[must_use]
pub fn is_wildcard_context(line: &str, match_start: usize) -> bool {
    // Use get() to safely handle potential mid-codepoint byte offsets
    let before = match line.get(..match_start) {
        Some(s) => s.to_lowercase(),
        None => return false, // Invalid byte offset, assume not wildcard context
    };

    for ctx in WILDCARD_ALLOWED_CONTEXTS {
        if before.contains(ctx) {
            return true;
        }
    }

    // Also check for $filter anywhere in the line
    if line.to_lowercase().contains("$filter") {
        return true;
    }

    false
}

/// Check if the GTS identifier is in a "bad example" context
#[must_use]
pub fn is_bad_example_context(line: &str, prev_lines: &[&str]) -> bool {
    let line_lower = line.to_lowercase();

    // Check current line
    for ctx in SKIP_VALIDATION_CONTEXTS {
        if line_lower.contains(ctx) {
            return true;
        }
    }

    // Check previous lines (last 3)
    for prev_line in prev_lines.iter().rev().take(3) {
        let prev_lower = prev_line.to_lowercase();
        for ctx in SKIP_VALIDATION_CONTEXTS {
            if prev_lower.contains(ctx) {
                return true;
            }
        }
    }

    false
}

/// Validate a single GTS segment like 'x.core.modkit.plugin.v1'
pub fn validate_gts_segment(segment: &str) -> Result<(), String> {
    if segment.is_empty() {
        return Ok(()); // Empty segments are ok (trailing ~)
    }

    // Check for invalid characters
    if segment.contains('-') {
        return Err(format!("Hyphen not allowed in segment: '{segment}'"));
    }

    let parts: Vec<&str> = segment.split('.').collect();

    // Must have 5 components: vendor.org.package.type.version
    // But version can be v1, v1.0, v1.2.3, etc.
    if parts.len() < 5 {
        return Err(format!(
            "Segment must have 5 components (vendor.org.package.type.version), got {}: '{segment}'",
            parts.len()
        ));
    }

    // The 5th component must start with 'v' (version)
    if !parts[4].starts_with('v') {
        return Err(format!(
            "Version must start with 'v' (e.g., v1, v1.0): '{segment}'"
        ));
    }

    // Validate version format
    let version_part = &parts[4][1..]; // Remove 'v' prefix
    if version_part.is_empty() {
        return Err(format!("Version number missing after 'v': '{segment}'"));
    }

    // Version parts must be numeric
    let version_components: Vec<&str> = if parts.len() > 5 {
        // v1.2.3 case: version spans multiple dot-separated parts
        std::iter::once(version_part)
            .chain(parts[5..].iter().copied())
            .collect()
    } else {
        vec![version_part]
    };

    for vc in &version_components {
        if vc.parse::<u32>().is_err() {
            return Err(format!("Version components must be numeric: '{segment}'"));
        }
    }

    // Validate component format (lowercase alphanumeric + underscore)
    for (i, part) in parts[..4].iter().enumerate() {
        if part.is_empty() {
            return Err(format!("Empty component at position {i}: '{segment}'"));
        }
        if !part
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(format!(
                "Components must be lowercase alphanumeric with underscores only: '{segment}'"
            ));
        }
    }

    Ok(())
}

/// Validate a complete GTS identifier and optionally check vendor
pub fn validate_gts_id(
    gts_id: &str,
    expected_vendor: Option<&str>,
    allow_wildcards: bool,
) -> Vec<String> {
    let mut errors = Vec::new();
    let original = gts_id;

    // Normalize: remove quotes if present
    let gts_id = gts_id.trim().trim_matches(|c| c == '"' || c == '\'');

    if !gts_id.starts_with("gts.") {
        return vec![format!("Must start with 'gts.': '{original}'")];
    }

    // Check for wildcards
    if gts_id.contains('*') && !allow_wildcards {
        return vec![format!(
            "Wildcards not allowed outside pattern contexts: '{original}'"
        )];
    }

    // If wildcards are present and allowed, do basic structure check only
    if gts_id.contains('*') {
        if let Some(expected) = expected_vendor {
            let rest = &gts_id[4..]; // Remove 'gts.' prefix
            if let Some(first_seg) = rest.split('~').next()
                && let Some(vendor) = first_seg.split('.').next()
                && !vendor.contains('*')
                && vendor != expected
                && !is_example_vendor(vendor)
            {
                return vec![format!(
                    "Vendor mismatch: expected '{expected}', found '{vendor}' in '{original}'"
                )];
            }
        }
        return vec![];
    }

    // Try to validate using the gts library
    if let Ok(parsed) = GtsID::new(gts_id) {
        // Check vendor if specified
        if let Some(expected) = expected_vendor
            && let Some(first_segment) = parsed.gts_id_segments.first()
        {
            let actual_vendor = &first_segment.vendor;
            // Skip vendor check for example/placeholder vendors
            if actual_vendor != expected && !is_example_vendor(actual_vendor) {
                errors.push(format!(
                        "Vendor mismatch: expected '{expected}', found '{actual_vendor}' in '{original}'"
                    ));
            }
        }
    } else {
        // The gts library failed to parse, do our own validation
        // to provide more specific error messages
        let rest = &gts_id[4..]; // Remove 'gts.' prefix
        let segments: Vec<&str> = rest.split('~').collect();
        let non_empty_segments: Vec<&str> =
            segments.iter().filter(|s| !s.is_empty()).copied().collect();

        if non_empty_segments.is_empty() {
            errors.push(format!("No segments found after 'gts.': '{original}'"));
            return errors;
        }

        for seg in &non_empty_segments {
            if let Err(e) = validate_gts_segment(seg) {
                errors.push(e);
            }
        }

        // Schema IDs (single segment) must end with ~
        if non_empty_segments.len() == 1 && !gts_id.ends_with('~') {
            errors.push(format!("Schema ID must end with '~': '{original}'"));
        }

        // If gts library rejected it but our validation passed, add a generic error
        if errors.is_empty() {
            errors.push(format!("Invalid GTS ID format: '{original}'"));
        }

        // Even if gts library failed, still check vendor
        if let Some(expected) = expected_vendor
            && let Some(first_seg) = non_empty_segments.first()
        {
            let parts: Vec<&str> = first_seg.split('.').collect();
            if let Some(vendor) = parts.first()
                && *vendor != expected
                && !is_example_vendor(vendor)
            {
                errors.push(format!(
                    "Vendor mismatch: expected '{expected}', found '{vendor}' in '{original}'"
                ));
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_segment_standard() {
        assert!(validate_gts_segment("x.core.modkit.plugin.v1").is_ok());
    }

    #[test]
    fn test_valid_segment_with_underscores() {
        assert!(validate_gts_segment("my_vendor.my_org.my_package.my_type.v1").is_ok());
    }

    #[test]
    fn test_valid_segment_version_with_minor() {
        assert!(validate_gts_segment("x.core.modkit.plugin.v1.2").is_ok());
    }

    #[test]
    fn test_invalid_segment_hyphen() {
        let result = validate_gts_segment("my-vendor.org.pkg.type.v1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Hyphen"));
    }

    #[test]
    fn test_invalid_segment_too_few_components() {
        let result = validate_gts_segment("x.core.plugin.v1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("5 components"));
    }

    #[test]
    fn test_validate_gts_id_valid() {
        let errors = validate_gts_id("gts.x.core.modkit.plugin.v1~", None, false);
        assert!(errors.is_empty(), "Unexpected errors: {errors:?}");
    }

    #[test]
    fn test_validate_gts_id_vendor_match() {
        let errors = validate_gts_id("gts.x.core.modkit.plugin.v1~", Some("x"), false);
        assert!(errors.is_empty(), "Unexpected errors: {errors:?}");
    }

    #[test]
    fn test_validate_gts_id_vendor_mismatch() {
        let errors = validate_gts_id("gts.hx.core.modkit.plugin.v1~", Some("x"), false);
        assert!(!errors.is_empty());
        assert!(errors[0].contains("Vendor mismatch"));
    }

    #[test]
    fn test_validate_gts_id_example_vendor_tolerated() {
        // Example vendors like 'acme' and 'globex' should be tolerated
        let errors = validate_gts_id("gts.acme.core.events.user_created.v1~", Some("x"), false);
        assert!(
            errors.is_empty(),
            "Example vendor 'acme' should be tolerated: {errors:?}"
        );

        let errors = validate_gts_id("gts.globex.core.events.order.v1~", Some("x"), false);
        assert!(
            errors.is_empty(),
            "Example vendor 'globex' should be tolerated: {errors:?}"
        );
    }

    #[test]
    fn test_is_example_vendor() {
        assert!(is_example_vendor("acme"));
        assert!(is_example_vendor("globex"));
        assert!(is_example_vendor("example"));
        assert!(is_example_vendor("demo"));
        assert!(is_example_vendor("test"));
        assert!(!is_example_vendor("x"));
        assert!(!is_example_vendor("hx"));
        assert!(!is_example_vendor("cf"));
    }

    #[test]
    #[allow(unknown_lints, de0901_gts_string_pattern)] // Testing wildcard handling
    fn test_validate_gts_id_wildcard_allowed() {
        let errors = validate_gts_id("gts.x.*", None, true);
        assert!(errors.is_empty());
    }

    #[test]
    #[allow(unknown_lints, de0901_gts_string_pattern)] // Testing wildcard rejection
    fn test_validate_gts_id_wildcard_not_allowed() {
        let errors = validate_gts_id("gts.x.*", None, false);
        assert!(!errors.is_empty());
        assert!(errors[0].contains("Wildcards"));
    }

    #[test]
    fn test_is_wildcard_context() {
        assert!(is_wildcard_context(
            "$filter=type_id eq 'gts.x.*'",
            "$filter=type_id eq '".len()
        ));
        assert!(is_wildcard_context(
            "Use this pattern: gts.x.core.*",
            "Use this pattern: ".len()
        ));
        assert!(!is_wildcard_context(
            "The type gts.x.core.type.v1~",
            "The type ".len()
        ));
    }

    #[test]
    fn test_is_bad_example_context() {
        assert!(is_bad_example_context("Invalid: gts.bad.id", &[]));
        assert!(is_bad_example_context("\u{274c} gts.x.y.z.a.v1~", &[]));
        assert!(!is_bad_example_context(
            "The correct format is gts.x.core.type.v1~",
            &[]
        ));
    }
}
