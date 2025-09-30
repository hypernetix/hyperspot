use users_info::errors::ErrorCode;

#[test]
fn error_code_has_correct_status() {
    assert_eq!(ErrorCode::users_info_user_not_found_v1.status(), 404);
    assert_eq!(ErrorCode::users_info_user_email_conflict_v1.status(), 409);
    assert_eq!(ErrorCode::users_info_user_validation_v1.status(), 422);
    assert_eq!(ErrorCode::users_info_internal_database_v1.status(), 500);
}

#[test]
fn error_code_to_problem_works() {
    let problem = ErrorCode::users_info_user_not_found_v1.to_problem("User not found");

    assert_eq!(problem.status, 404);
    assert_eq!(problem.title, "User Not Found");
    assert_eq!(problem.code, "users_info.user.not_found.v1");
    assert_eq!(problem.detail, "User not found");
    assert_eq!(
        problem.type_url,
        "https://errors.example.com/users_info.user.not_found.v1"
    );
}

#[test]
fn error_code_def_is_consistent() {
    let def = ErrorCode::users_info_user_email_conflict_v1.def();

    assert_eq!(def.status, 409);
    assert_eq!(def.title, "Email Already Exists");
    assert_eq!(def.code, "users_info.user.email_conflict.v1");
    assert_eq!(
        def.type_url,
        "https://errors.example.com/users_info.user.email_conflict.v1"
    );
}

#[test]
fn all_error_codes_have_valid_status() {
    // Test all error codes to ensure they have valid HTTP status codes
    let codes = [
        ErrorCode::users_info_user_not_found_v1,
        ErrorCode::users_info_user_email_conflict_v1,
        ErrorCode::users_info_user_invalid_email_v1,
        ErrorCode::users_info_user_validation_v1,
        ErrorCode::users_info_odata_invalid_filter_v1,
        ErrorCode::users_info_odata_invalid_orderby_v1,
        ErrorCode::users_info_odata_invalid_cursor_v1,
        ErrorCode::users_info_internal_database_v1,
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
fn to_response_attaches_context() {
    let resp = ErrorCode::users_info_user_not_found_v1.to_response(
        "User not found",
        "/users/123",
        Some("trace-1".to_string()),
    );

    assert_eq!(resp.0.instance, "/users/123");
    assert_eq!(resp.0.trace_id.as_deref(), Some("trace-1"));
    assert_eq!(resp.0.status, 404);
    assert_eq!(resp.0.detail, "User not found");
}

#[test]
fn validation_errors_use_422() {
    assert_eq!(ErrorCode::users_info_user_validation_v1.status(), 422);
    assert_eq!(ErrorCode::users_info_odata_invalid_filter_v1.status(), 422);
    assert_eq!(ErrorCode::users_info_odata_invalid_orderby_v1.status(), 422);
    assert_eq!(ErrorCode::users_info_odata_invalid_cursor_v1.status(), 422);
}

#[test]
fn invalid_email_remains_400() {
    assert_eq!(ErrorCode::users_info_user_invalid_email_v1.status(), 400);
}

#[test]
fn problem_from_catalog_macro_works() {
    // This would be a compile-time test, but we can at least verify it compiles
    // by using it in a test context
    let _p1 = users_info::problem_from_catalog!("users_info.user.not_found.v1");
    let _p2 = users_info::problem_from_catalog!(
        "users_info.user.email_conflict.v1",
        "test@example.com already exists"
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
