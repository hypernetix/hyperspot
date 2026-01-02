#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for error handling and conversions
//!
//! These tests verify domain error conversions including HTTP Problem mapping and anyhow integration.

use axum::http::StatusCode;
use modkit::api::problem::Problem;
use nodes_registry::api::rest::error::domain_error_to_problem;
use nodes_registry::domain::error::DomainError;

#[test]
fn test_error_conversion_mapping() {
    let test_id = uuid::Uuid::new_v4();

    // Test all error types with their expected mappings
    let test_cases = vec![
        (
            DomainError::NodeNotFound(test_id),
            StatusCode::NOT_FOUND,
            "NODES_NOT_FOUND",
            test_id.to_string(),
            "/test/nodes",
        ),
        (
            DomainError::SysInfoCollectionFailed("Failed to read CPU info".to_owned()),
            StatusCode::INTERNAL_SERVER_ERROR,
            "SYSINFO_COLLECTION_FAILED",
            "Failed to read CPU info".to_owned(),
            "/test/sysinfo",
        ),
        (
            DomainError::SysCapCollectionFailed("GPU detection failed".to_owned()),
            StatusCode::INTERNAL_SERVER_ERROR,
            "SYSCAP_COLLECTION_FAILED",
            "GPU detection failed".to_owned(),
            "/test/syscap",
        ),
        (
            DomainError::InvalidInput("Invalid capability key format".to_owned()),
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Invalid capability key format".to_owned(),
            "/test/validate",
        ),
        (
            DomainError::Internal("Database connection lost".to_owned()),
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "Database connection lost".to_owned(),
            "/test/internal",
        ),
    ];

    for (error, expected_status, expected_code, expected_detail_content, instance_path) in
        test_cases
    {
        let problem = domain_error_to_problem(error, instance_path);

        assert_eq!(problem.status, expected_status, "Status code should match");
        assert_eq!(problem.code, expected_code, "Error code should match");
        assert!(
            problem.detail.contains(&expected_detail_content),
            "Detail should contain error info"
        );
        assert_eq!(
            problem.instance, instance_path,
            "Instance path should be preserved"
        );
        assert!(!problem.type_url.is_empty(), "Type URL should not be empty");
        assert!(
            problem
                .type_url
                .starts_with("https://errors.hyperspot.com/"),
            "Type URL should have correct prefix"
        );
    }
}

#[test]
fn test_error_into_problem_trait() {
    let node_id = uuid::Uuid::new_v4();
    let error = DomainError::NodeNotFound(node_id);

    // Test From<DomainError> for Problem
    let problem: Problem = error.into();

    assert_eq!(problem.status, StatusCode::NOT_FOUND);
    assert!(problem.detail.contains(&node_id.to_string()));
    // Default instance should be "/"
    assert_eq!(problem.instance, "/");
}

#[test]
fn test_domain_error_from_anyhow() {
    let anyhow_err = anyhow::anyhow!("something went wrong");
    let domain_err: DomainError = anyhow_err.into();

    match domain_err {
        DomainError::Internal(msg) => {
            assert!(msg.contains("something went wrong"));
        }
        _ => panic!("Expected Internal error"),
    }
}
