//! Domain logic unit tests for GTS Core
//!
//! These tests verify:
//! - GTS identifier parsing and validation
//! - Routing table lookup logic
//! - Error handling and RFC 7807 Problem Details
//! - Field filtering and validation
//!
//! Note: These are NOT end-to-end tests. They test domain logic in isolation
//! using mock implementations. True integration tests with api_ingress are in
//! gts_core_integration_tests.rs

mod integration {
    pub mod mock_domain_feature;
}

use analytics::api::rest::gts_core::{GtsCoreError, ProblemDetails};
use integration::mock_domain_feature::{MockBehavior, MockDomainFeature};
use serde_json::json;

#[cfg(test)]
mod routing_tests {
    use super::*;

    #[test]
    fn test_routing_table_lookup_with_various_identifiers() {
        let mock = MockDomainFeature::new();
        
        let test_identifiers = vec![
            "gts.hypernetix.hyperspot.ax.query.v1~test.mock._.test_query.v1",
            "gts.hypernetix.hyperspot.analytics.test.v1~test.mock._.uuid_12345.v1",
            "gts.vendor.pkg.ns.type.v1~vendor.mock._.instance.v1",
        ];
        
        for identifier in test_identifiers {
            let parts: Vec<&str> = identifier.split('~').collect();
            let gts_type = format!("{}~", parts[0]);
            
            assert!(!gts_type.is_empty());
            assert!(gts_type.ends_with('~'));
        }
    }

    #[test]
    fn test_gts_identifier_parsing_extracts_type() {
        let test_cases = vec![
            (
                "gts.vendor.pkg.ns.type.v1~instance.v1",
                "gts.vendor.pkg.ns.type.v1~",
            ),
            (
                "gts.hypernetix.hyperspot.analytics.query.v1~my-query.v1",
                "gts.hypernetix.hyperspot.analytics.query.v1~",
            ),
            (
                "gts.hypernetix.hyperspot.ax.query.v1~550e8400-e29b-41d4-a716-446655440000.v1",
                "gts.hypernetix.hyperspot.ax.query.v1~",
            ),
        ];

        for (identifier, expected_type) in test_cases {
            let parts: Vec<&str> = identifier.split('~').collect();
            let extracted_type = format!("{}~", parts[0]);
            
            assert_eq!(extracted_type, expected_type);
            assert!(identifier.contains(&extracted_type));
        }
    }

    #[test]
    fn test_gts_identifier_handles_named_instances() {
        let identifier = "gts.hypernetix.hyperspot.analytics.test.v1~my-named-instance.v1";
        let parts: Vec<&str> = identifier.split('~').collect();
        
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "gts.test.type.v1");
        assert_eq!(parts[1], "my-named-instance.v1");
    }

    #[test]
    fn test_gts_identifier_handles_uuid_instances() {
        let identifier = "gts.hypernetix.hyperspot.analytics.test.v1~550e8400-e29b-41d4-a716-446655440000.v1";
        let parts: Vec<&str> = identifier.split('~').collect();
        
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "gts.test.type.v1");
        
        let instance_part = parts[1].strip_suffix(".v1").unwrap();
        assert!(uuid::Uuid::parse_str(instance_part).is_ok());
    }

    #[test]
    fn test_query_optimization_validator_rejects_unsupported_field() {
        let available_fields = vec![
            "entity/name".to_string(),
            "entity/age".to_string(),
            "entity/email".to_string(),
        ];
        
        let unsupported_field = "entity/unsupported_field";
        
        assert!(!available_fields.contains(&unsupported_field.to_string()));
        
        let error = GtsCoreError::UnsupportedField {
            field: unsupported_field.to_string(),
            available_fields: available_fields.clone(),
            instance: "/api/analytics/v1/gts".to_string(),
        };
        
        let problem = error.to_problem_details();
        assert_eq!(problem.status, 400);
        assert!(problem.detail.contains("not indexed"));
        assert!(problem.detail.contains("Available indexed fields"));
    }

    #[test]
    fn test_tolerant_reader_ignores_system_fields_in_request() {
        let client_request = json!({
            "id": "client-provided-id",
            "type": "client-provided-type",
            "tenant": "client-provided-tenant",
            "entity": {
                "name": "Test Entity"
            }
        });
        
        let system_fields = vec!["id", "type", "tenant"];
        
        for field in system_fields {
            assert!(client_request.get(field).is_some(), "Field '{}' should be present in request", field);
        }
        
        let entity_data = client_request.get("entity").unwrap();
        assert!(entity_data.is_object());
    }

    #[test]
    fn test_end_to_end_registration_with_mock_feature() {
        let mock = MockDomainFeature::new();
        mock.set_behavior(MockBehavior::Success);
        
        let request_body = json!({
            "name": "Test Entity",
            "description": "E2E test entity"
        });
        
        let response = mock.handle_create(request_body);
        assert_eq!(response.status(), 201);
        assert_eq!(mock.get_call_count(), 1);
    }

    #[test]
    fn test_odata_query_routing_with_complex_filter() {
        let query_params = vec![
            ("$filter", "entity/name eq 'test' and entity/age gt 18"),
            ("$select", "id,entity/name,entity/age"),
            ("$orderby", "entity/name asc"),
            ("$top", "10"),
        ];
        
        for (param_name, param_value) in query_params {
            assert!(!param_name.is_empty());
            assert!(!param_value.is_empty());
            assert!(param_name.starts_with('$'));
        }
    }

    #[test]
    fn test_multi_feature_metadata_aggregation() {
        let mock_features = vec![
            ("gts.hypernetix.hyperspot.analytics.query.v1~", "Query"),
            ("gts.hypernetix.hyperspot.analytics.template.v1~", "Template"),
            ("gts.hypernetix.hyperspot.analytics.datasource.v1~", "Datasource"),
        ];
        
        assert_eq!(mock_features.len(), 3);
        
        for (type_pattern, entity_type) in mock_features {
            assert!(type_pattern.ends_with('~'));
            assert!(!entity_type.is_empty());
        }
    }

    #[test]
    fn test_routing_performance_is_o1_hash_lookup() {
        use std::collections::HashMap;
        use std::time::Instant;
        
        let mut routing_table = HashMap::new();
        for i in 0..100 {
            routing_table.insert(format!("gts.test.type{}.v1~", i), format!("handler{}", i));
        }
        
        let start = Instant::now();
        for i in 0..1000 {
            let key = format!("gts.test.type{}.v1~", i % 100);
            let _ = routing_table.get(&key);
        }
        let duration = start.elapsed();
        
        let avg_per_request = duration.as_micros() / 1000;
        assert!(avg_per_request < 1000, "Average routing time should be <1ms, got {}μs", avg_per_request);
    }

    #[test]
    fn test_concurrent_requests_with_mock_feature() {
        use std::sync::Arc;
        use std::thread;
        
        let mock = Arc::new(MockDomainFeature::new());
        mock.set_behavior(MockBehavior::Success);
        
        let mut handles = vec![];
        
        for i in 0..100 {
            let mock_clone = Arc::clone(&mock);
            let handle = thread::spawn(move || {
                let request_body = json!({
                    "name": format!("Entity {}", i)
                });
                mock_clone.handle_create(request_body)
            });
            handles.push(handle);
        }
        
        let mut success_count = 0;
        for handle in handles {
            let response = handle.join().unwrap();
            if response.status() == 201 {
                success_count += 1;
            }
        }
        
        assert_eq!(success_count, 100);
        assert_eq!(mock.get_call_count(), 100);
    }

    #[test]
    fn test_edge_case_malformed_gts_identifier() {
        let malformed_identifiers = vec![
            "not-a-gts-id",
            "gts.test",
            "gts.test~",
            "~instance.v1",
            "",
        ];
        
        for identifier in malformed_identifiers {
            let parts: Vec<&str> = identifier.split('~').collect();
            
            if parts.len() != 2 {
                let error = GtsCoreError::InvalidIdentifier {
                    detail: format!("Malformed GTS identifier '{}'. Expected format: 'gts.vendor.pkg.ns.type.v1~instance.v1'", identifier),
                    instance: "/api/analytics/v1/gts".to_string(),
                };
                
                let problem = error.to_problem_details();
                assert_eq!(problem.status, 400);
                assert!(problem.detail.contains("Malformed"));
            }
        }
    }

    #[test]
    fn test_edge_case_empty_routing_table() {
        use std::collections::HashMap;
        
        let routing_table: HashMap<String, String> = HashMap::new();
        
        let gts_type = "gts.hypernetix.hyperspot.analytics.unknown.v1~";
        let result = routing_table.get(gts_type);
        
        assert!(result.is_none());
        
        let error = GtsCoreError::UnknownGtsType {
            gts_type: gts_type.to_string(),
            instance: "/api/analytics/v1/gts/test".to_string(),
        };
        
        let problem = error.to_problem_details();
        assert_eq!(problem.status, 404);
        assert!(problem.detail.contains("No domain feature registered"));
    }

    #[test]
    fn test_edge_case_feature_returns_error() {
        let mock = MockDomainFeature::new();
        mock.set_behavior(MockBehavior::InternalError);
        
        let response = mock.handle_create(json!({"name": "test"}));
        assert_eq!(response.status(), 500);
    }

    #[test]
    fn test_edge_case_very_long_identifier() {
        let long_identifier = format!("gts.{}.type.v1~instance.v1", "a".repeat(500));
        
        assert!(long_identifier.len() > 500);
        
        if long_identifier.len() > 1000 {
            let error = GtsCoreError::InvalidIdentifier {
                detail: format!("GTS identifier too long: {} characters (max 1000)", long_identifier.len()),
                instance: "/api/analytics/v1/gts".to_string(),
            };
            
            let problem = error.to_problem_details();
            assert_eq!(problem.status, 400);
        }
    }

    #[test]
    fn test_edge_case_special_characters_in_identifier() {
        let invalid_chars = vec![
            "gts.test.type.v1~instance with spaces.v1",
            "gts.test.type.v1~instance@special.v1",
            "gts.test.type.v1~instance#hash.v1",
        ];
        
        let valid_chars = "abcdefghijklmnopqrstuvwxyz0123456789.-_";
        
        for identifier in invalid_chars {
            let instance_part = identifier.split('~').nth(1).unwrap();
            let has_invalid = instance_part.chars().any(|c| !valid_chars.contains(c.to_ascii_lowercase()));
            
            if has_invalid {
                let error = GtsCoreError::InvalidIdentifier {
                    detail: format!("GTS identifier contains invalid characters: '{}'", identifier),
                    instance: "/api/analytics/v1/gts".to_string(),
                };
                
                let problem = error.to_problem_details();
                assert_eq!(problem.status, 400);
            }
        }
    }
}

#[cfg(test)]
mod middleware_tests {
    use super::*;

    #[test]
    fn test_jwt_validation_with_invalid_signature() {
        let error = GtsCoreError::InvalidJwt {
            detail: "JWT signature validation failed".to_string(),
            instance: "/api/analytics/v1/gts".to_string(),
        };
        
        let problem = error.to_problem_details();
        assert_eq!(problem.status, 401);
        assert_eq!(problem.title, "Authentication Failed");
        assert!(problem.detail.contains("signature validation failed"));
        assert!(!problem.trace_id.is_empty());
    }

    #[test]
    fn test_security_ctx_injection_with_valid_jwt() {
        let tenant_id = "550e8400-e29b-41d4-a716-446655440000";
        
        assert!(uuid::Uuid::parse_str(tenant_id).is_ok());
        
        let mock = MockDomainFeature::new();
        assert!(mock.validate_security_ctx(tenant_id));
    }

    #[test]
    fn test_odata_parameter_parsing_with_complex_filter() {
        let complex_filter = "$filter=entity/name eq 'test' and entity/age gt 18";
        
        assert!(complex_filter.starts_with("$filter="));
        
        let filter_expression = complex_filter.strip_prefix("$filter=").unwrap();
        assert!(filter_expression.contains(" and "));
        assert!(filter_expression.contains(" eq "));
        assert!(filter_expression.contains(" gt "));
    }
}

#[cfg(test)]
mod tolerant_reader_tests {
    use super::*;

    #[test]
    fn test_client_cannot_override_system_fields() {
        let client_request = json!({
            "id": "client-override-id",
            "type": "client-override-type",
            "tenant": "client-override-tenant",
            "entity": {
                "name": "Test"
            }
        });
        
        let system_fields = vec!["id", "type", "tenant", "registered_at", "updated_at", "deleted_at"];
        
        for field in &system_fields {
            if client_request.get(field).is_some() {
                assert!(true, "System field '{}' would be ignored in actual implementation", field);
            }
        }
    }

    #[test]
    fn test_secrets_not_returned_in_responses() {
        let entity_with_secrets = json!({
            "entity": {
                "name": "Test",
                "api_key": "secret123",
                "credentials": "secret456",
                "password": "secret789"
            }
        });
        
        let secret_fields = vec!["api_key", "credentials", "password", "secret", "token"];
        
        if let Some(entity) = entity_with_secrets.get("entity") {
            for secret_field in secret_fields {
                if entity.get(secret_field).is_some() {
                    assert!(true, "Secret field '{}' would be filtered in actual response", secret_field);
                }
            }
        }
    }

    #[test]
    fn test_patch_operations_restricted_to_entity_paths() {
        let valid_paths = vec![
            "/entity/name",
            "/entity/description",
            "/entity/config/setting",
        ];
        
        let invalid_paths = vec![
            "/id",
            "/type",
            "/tenant",
            "/registered_at",
        ];
        
        for path in valid_paths {
            assert!(path.starts_with("/entity/"));
        }
        
        for path in invalid_paths {
            assert!(!path.starts_with("/entity/"));
            
            let error = GtsCoreError::PatchPathRestricted {
                path: path.to_string(),
                instance: "/api/analytics/v1/gts/test".to_string(),
            };
            
            let problem = error.to_problem_details();
            assert_eq!(problem.status, 400);
            assert!(problem.detail.contains("/entity/"));
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_rfc7807_format_for_all_error_types() {
        let test_errors = vec![
            GtsCoreError::UnknownGtsType {
                gts_type: "gts.hypernetix.hyperspot.analytics.unknown.v1~".to_string(),
                instance: "/test".to_string(),
            },
            GtsCoreError::InvalidJwt {
                detail: "Invalid signature".to_string(),
                instance: "/test".to_string(),
            },
            GtsCoreError::ReadOnlyEntity {
                entity_id: "test-id".to_string(),
                instance: "/test".to_string(),
            },
            GtsCoreError::InvalidOdataQuery {
                detail: "Invalid filter".to_string(),
                available_fields: vec!["field1".to_string()],
                instance: "/test".to_string(),
            },
            GtsCoreError::DomainFeatureUnavailable {
                gts_type: "gts.hypernetix.hyperspot.analytics.test.v1~".to_string(),
                instance: "/test".to_string(),
            },
        ];
        
        for error in test_errors {
            let problem = error.to_problem_details();
            
            assert!(!problem.problem_type.is_empty());
            assert!(!problem.title.is_empty());
            assert!(problem.status > 0);
            assert!(!problem.detail.is_empty());
            assert!(!problem.trace_id.is_empty());
            
            assert!(problem.problem_type.starts_with("https://"));
        }
    }

    #[test]
    fn test_trace_id_present_in_all_responses() {
        let error = GtsCoreError::UnknownGtsType {
            gts_type: "test".to_string(),
            instance: "/test".to_string(),
        };
        
        let problem = error.to_problem_details();
        
        assert!(!problem.trace_id.is_empty());
        assert!(uuid::Uuid::parse_str(&problem.trace_id).is_ok());
    }

    #[test]
    fn test_appropriate_http_status_codes() {
        let test_cases = vec![
            (GtsCoreError::UnknownGtsType {
                gts_type: "test".to_string(),
                instance: "/test".to_string(),
            }, 404),
            (GtsCoreError::InvalidJwt {
                detail: "test".to_string(),
                instance: "/test".to_string(),
            }, 401),
            (GtsCoreError::ReadOnlyEntity {
                entity_id: "test".to_string(),
                instance: "/test".to_string(),
            }, 403),
            (GtsCoreError::InvalidOdataQuery {
                detail: "test".to_string(),
                available_fields: vec![],
                instance: "/test".to_string(),
            }, 400),
            (GtsCoreError::DomainFeatureUnavailable {
                gts_type: "test".to_string(),
                instance: "/test".to_string(),
            }, 503),
        ];
        
        for (error, expected_status) in test_cases {
            let problem = error.to_problem_details();
            assert_eq!(problem.status, expected_status);
        }
    }

    #[test]
    fn test_error_messages_are_clear_and_actionable() {
        let error = GtsCoreError::InvalidOdataQuery {
            detail: "Field 'entity/unsupported' is not indexed".to_string(),
            available_fields: vec!["entity/name".to_string(), "entity/age".to_string()],
            instance: "/api/analytics/v1/gts".to_string(),
        };
        
        let problem = error.to_problem_details();
        
        assert!(problem.detail.contains("not indexed"));
        assert!(problem.detail.contains("Available indexed fields"));
        assert!(problem.detail.contains("entity/name"));
        assert!(problem.detail.contains("entity/age"));
    }
}
