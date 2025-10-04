use users_info::errors::ErrorCode;

#[test]
fn error_code_has_correct_status() {
    assert_eq!(ErrorCode::example1_user_not_found_v1().status(), 404);
    assert_eq!(
        ErrorCode::gts_hx_core_errors_err_v1_hx_example1_user_email_conflict_v1.status(),
        409
    );
    assert_eq!(
        ErrorCode::gts_hx_core_errors_err_v1_hx_example1_user_validation_v1.status(),
        422
    );
    assert_eq!(
        ErrorCode::gts_hx_core_errors_err_v1_hx_example1_user_internal_database_v1.status(),
        500
    );
}

#[test]
fn error_code_to_problem_works() {
    let problem = ErrorCode::example1_user_not_found_v1().to_problem("User not found");

    assert_eq!(problem.status, 404);
    assert_eq!(problem.title, "User Not Found");
    assert_eq!(
        problem.code,
        "gts.hx.core.errors.err.v1~hx.example1.user.not_found.v1"
    );
    assert_eq!(problem.detail, "User not found");
    assert_eq!(
        problem.type_url,
        "https://errors.example.com/gts.hx.core.errors.err.v1~hx.example1.user.not_found.v1"
    );
}

#[test]
fn error_code_def_is_consistent() {
    let def = ErrorCode::gts_hx_core_errors_err_v1_hx_example1_user_email_conflict_v1.def();

    assert_eq!(def.status, 409);
    assert_eq!(def.title, "Email Already Exists");
    assert_eq!(
        def.code,
        "gts.hx.core.errors.err.v1~hx.example1.user.email_conflict.v1"
    );
    assert_eq!(
        def.type_url,
        "https://errors.example.com/gts.hx.core.errors.err.v1~hx.example1.user.email_conflict.v1"
    );
}

#[test]
fn all_error_codes_have_valid_status() {
    // Test all error codes to ensure they have valid HTTP status codes
    let codes = [
        ErrorCode::gts_hx_core_errors_err_v1_hx_example1_user_not_found_v1,
        ErrorCode::gts_hx_core_errors_err_v1_hx_example1_user_email_conflict_v1,
        ErrorCode::example1_user_invalid_email_v1(),
        ErrorCode::gts_hx_core_errors_err_v1_hx_example1_user_validation_v1,
        ErrorCode::gts_hx_core_errors_err_v1_hx_example1_user_invalid_email_v1,
        ErrorCode::gts_hx_core_errors_err_v1_hx_example1_user_internal_database_v1,
    ];

    for code in &codes {
        let status = code.status();
        assert!(
            (100..=599).contains(&status),
            "Invalid status code: {}",
            status
        );
    }
}

#[test]
fn with_context_attaches_instance_and_trace() {
    let problem = ErrorCode::gts_hx_core_errors_err_v1_hx_example1_user_not_found_v1.with_context(
        "User not found",
        "/users/123",
        Some("trace-1".to_string()),
    );

    assert_eq!(problem.instance, "/users/123");
    assert_eq!(problem.trace_id.as_deref(), Some("trace-1"));
    assert_eq!(problem.status, 404);
    assert_eq!(problem.detail, "User not found");
}

#[test]
fn validation_errors_use_422() {
    assert_eq!(
        ErrorCode::gts_hx_core_errors_err_v1_hx_example1_user_validation_v1.status(),
        422
    );
}

#[test]
fn invalid_email_remains_400() {
    assert_eq!(ErrorCode::example1_user_invalid_email_v1().status(), 400);
}

#[test]
fn short_accessor_with_alias_works() {
    // Test the explicit alias accessor
    let code = ErrorCode::example1_user_email_conflict_v1();
    assert_eq!(code.status(), 409);
    assert_eq!(
        code.def().code,
        "gts.hx.core.errors.err.v1~hx.example1.user.email_conflict.v1"
    );
}

#[test]
fn short_accessor_derived_works() {
    // Test the auto-derived short accessor
    let code = ErrorCode::example1_user_invalid_email_v1();
    assert_eq!(code.status(), 400);
    assert_eq!(
        code.def().code,
        "gts.hx.core.errors.err.v1~hx.example1.user.invalid_email.v1"
    );
}

// Note: To test compile-time rejection of unknown codes, you would need
// a separate compile-fail test (trybuild), like:
//
// #[test]
// fn unknown_code_fails_to_compile() {
//     let t = trybuild::TestCases::new();
//     t.compile_fail("tests/ui/unknown_error_code.rs");
// }
