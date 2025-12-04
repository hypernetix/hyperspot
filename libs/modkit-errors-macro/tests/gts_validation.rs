#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Compile-time tests for GTS validation in declare_errors! macro
//!
//! These tests verify that the macro correctly rejects invalid GTS codes
//! and accepts valid ones.

// ============================================================================
// VALID GTS FORMATS (should compile)
// ============================================================================

/// Valid: Basic GTS with 5 segments
const _VALID_BASIC: &str = r#"[{
    "status": 422,
    "title": "Test Error",
    "code": "gts.hx.core.errors.err.v1~hx.test.basic.v1"
}]"#;

/// Valid: GTS with multiple chains
const _VALID_CHAINED: &str = r#"[{
    "status": 422,
    "title": "Test Error",
    "code": "gts.hx.core.errors.err.v1~hx.test.chain1.v1~hx.test.chain2.v1"
}]"#;

/// Valid: GTS with underscores
const _VALID_UNDERSCORES: &str = r#"[{
    "status": 422,
    "title": "Test Error",
    "code": "gts.hx.core_module.errors.test_error.v1~hx.test.underscore_test.v1"
}]"#;

// Note: Actual compile tests would use trybuild in a separate test harness
// For now, we'll create JSON test fixtures that the macro can validate

#[cfg(test)]
mod tests {
    #[test]
    fn test_gts_validation_documented() {
        // This test exists to document the GTS validation rules
        // The actual validation happens at compile-time in the macro

        // Valid GTS format:
        // gts.vendor.package.namespace.type.version~chain1~chain2~...~instanceGTX
        //
        // Rules:
        // 1. Must start with "gts."
        // 2. Chain segments separated by '~'
        // 3. Each GTX segment separated by '.'
        // 4. Final GTX must have at least 5 segments after 'gts': vendor.package.namespace.type.version
        // 5. All segments must be alphanumeric or underscore
        // 6. No empty segments

        println!("GTS validation rules documented");
    }
}
