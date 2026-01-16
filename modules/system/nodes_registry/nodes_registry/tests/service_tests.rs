#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Unit tests for the nodes registry service layer
//!
//! These tests verify service methods, error handling, and business logic.

use nodes_registry::domain::error::DomainError;
use nodes_registry::domain::service::Service;
use nodes_registry::SysCap;
use uuid::Uuid;

#[test]
fn test_service_initialization_creates_current_node() {
    let service = Service::new();

    let nodes = service.list_nodes();
    assert_eq!(
        nodes.len(),
        1,
        "Service should initialize with current node"
    );

    let node = &nodes[0];
    assert!(!node.hostname.is_empty(), "Node should have a hostname");
}

#[test]
fn test_get_node_returns_existing_node() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    let result = service.get_node(node_id);
    assert!(result.is_ok(), "Should successfully get existing node");

    let retrieved_node = result.unwrap();
    assert_eq!(retrieved_node.id, node_id);
    assert_eq!(retrieved_node.hostname, nodes[0].hostname);
}

#[test]
fn test_operations_on_nonexistent_node_return_error() {
    let service = Service::new();
    let nonexistent_id = Uuid::new_v4();

    // All operations on nonexistent nodes should return NodeNotFound error
    let operations_results = [
        service.get_node(nonexistent_id).err(),
        service.get_node_sysinfo(nonexistent_id).err(),
        service.get_node_syscap(nonexistent_id, false).err(),
        service.set_custom_syscap(nonexistent_id, vec![]).err(),
        service.remove_custom_syscap(nonexistent_id, vec![]).err(),
        service.clear_custom_syscap(nonexistent_id).err(),
    ];

    for (i, result) in operations_results.iter().enumerate() {
        assert!(result.is_some(), "Operation {i} should return error");
        match result.as_ref().unwrap() {
            DomainError::NodeNotFound(id) => {
                assert_eq!(
                    *id, nonexistent_id,
                    "Operation {i} should return correct node ID"
                );
            }
            _ => panic!("Operation {i} should return NodeNotFound error"),
        }
    }
}

#[test]
fn test_get_node_sysinfo_succeeds_for_existing_node() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    let result = service.get_node_sysinfo(node_id);
    assert!(
        result.is_ok(),
        "Should successfully collect sysinfo for existing node"
    );

    let sysinfo = result.unwrap();
    assert_eq!(sysinfo.node_id, node_id);
    assert!(!sysinfo.os.name.is_empty(), "Should have OS information");
    assert!(!sysinfo.cpu.model.is_empty(), "Should have CPU information");
    assert!(
        sysinfo.memory.total_bytes > 0,
        "Should have memory information"
    );
    assert!(
        !sysinfo.host.hostname.is_empty(),
        "Should have host information"
    );
}

#[test]
fn test_get_node_syscap_succeeds_for_existing_node() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    let result = service.get_node_syscap(node_id, false);
    assert!(
        result.is_ok(),
        "Should successfully collect syscap for existing node"
    );

    let syscap = result.unwrap();
    assert_eq!(syscap.node_id, node_id);
    assert!(
        !syscap.capabilities.is_empty(),
        "Should have system capabilities"
    );
}

#[test]
fn test_get_node_syscap_caches_result() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    // Add a custom capability with long TTL to test caching reliably
    let custom_cap = vec![SysCap {
        key: "custom.cache_test".to_owned(),
        category: "custom".to_owned(),
        name: "cache_test".to_owned(),
        display_name: "Cache Test".to_owned(),
        present: true,
        version: Some("1.0.0".to_owned()),
        amount: None,
        amount_dimension: None,
        details: None,
        cache_ttl_secs: 3600, // 1 hour
        fetched_at_secs: chrono::Utc::now().timestamp(),
    }];
    service.set_custom_syscap(node_id, custom_cap).unwrap();

    // First call - gets data (may refresh system caps)
    let result1 = service.get_node_syscap(node_id, false);
    assert!(result1.is_ok());
    let syscap1 = result1.unwrap();

    // Extract custom capability timestamp
    let custom1 = syscap1
        .capabilities
        .iter()
        .find(|c| c.key == "custom.cache_test")
        .expect("Should have custom capability");
    let fetched_at_1 = custom1.fetched_at_secs;

    // Second call immediately - should use cached data for custom capability
    let result2 = service.get_node_syscap(node_id, false);
    assert!(result2.is_ok());
    let syscap2 = result2.unwrap();

    // Custom capability should have same fetched_at_secs (proving it was cached)
    let custom2 = syscap2
        .capabilities
        .iter()
        .find(|c| c.key == "custom.cache_test")
        .expect("Should have custom capability");
    let fetched_at_2 = custom2.fetched_at_secs;

    assert_eq!(
        fetched_at_1, fetched_at_2,
        "Custom capability fetched_at_secs should match (cached)"
    );

    // Third call with force_refresh=true - should refresh all capabilities
    let result3 = service.get_node_syscap(node_id, true);
    assert!(result3.is_ok());
    let syscap3 = result3.unwrap();

    // After force refresh, collected_at should be newer
    assert!(
        syscap3.collected_at >= syscap2.collected_at,
        "Force refresh should update collected_at timestamp"
    );
}

#[test]
fn test_set_custom_syscap_adds_capabilities() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    let custom_caps = vec![
        SysCap {
            key: "custom.test.feature".to_owned(),
            category: "custom".to_owned(),
            name: "test_feature".to_owned(),
            display_name: "Test Feature".to_owned(),
            present: true,
            version: Some("1.0.0".to_owned()),
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 3600,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        },
        SysCap {
            key: "custom.another.feature".to_owned(),
            category: "custom".to_owned(),
            name: "another_feature".to_owned(),
            display_name: "Another Feature".to_owned(),
            present: false,
            version: None,
            amount: Some(42.0),
            amount_dimension: Some("units".to_owned()),
            details: None,
            cache_ttl_secs: 7200,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        },
    ];

    let result = service.set_custom_syscap(node_id, custom_caps.clone());
    assert!(result.is_ok(), "Should successfully set custom syscap");

    // Verify custom capabilities are merged
    let syscap = service.get_node_syscap(node_id, false).unwrap();
    let custom_keys: Vec<String> = custom_caps.iter().map(|c| c.key.clone()).collect();

    let found_custom_count = syscap
        .capabilities
        .iter()
        .filter(|c| custom_keys.contains(&c.key))
        .count();

    assert_eq!(
        found_custom_count, 2,
        "Should have both custom capabilities"
    );
}

#[test]
fn test_set_custom_syscap_overwrites_existing() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    // Set initial custom cap
    let cap_v1 = vec![SysCap {
        key: "custom.updatable".to_owned(),
        category: "custom".to_owned(),
        name: "updatable".to_owned(),
        display_name: "Updatable Feature".to_owned(),
        present: true,
        version: Some("1.0.0".to_owned()),
        amount: None,
        amount_dimension: None,
        details: None,
        cache_ttl_secs: 3600,
        fetched_at_secs: chrono::Utc::now().timestamp(),
    }];

    service.set_custom_syscap(node_id, cap_v1).unwrap();

    // Update with new version
    let cap_v2 = vec![SysCap {
        key: "custom.updatable".to_owned(),
        category: "custom".to_owned(),
        name: "updatable".to_owned(),
        display_name: "Updatable Feature".to_owned(),
        present: true,
        version: Some("2.0.0".to_owned()),
        amount: None,
        amount_dimension: None,
        details: None,
        cache_ttl_secs: 3600,
        fetched_at_secs: chrono::Utc::now().timestamp(),
    }];

    service.set_custom_syscap(node_id, cap_v2).unwrap();

    // Verify it was updated
    let syscap = service.get_node_syscap(node_id, false).unwrap();
    let custom_cap = syscap
        .capabilities
        .iter()
        .find(|c| c.key == "custom.updatable")
        .expect("Should find custom capability");

    assert_eq!(custom_cap.version, Some("2.0.0".to_owned()));
}

#[test]
fn test_remove_custom_syscap_removes_specified_keys() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    // Set multiple custom caps
    let custom_caps = vec![
        SysCap {
            key: "custom.keep".to_owned(),
            category: "custom".to_owned(),
            name: "keep".to_owned(),
            display_name: "Keep This".to_owned(),
            present: true,
            version: None,
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 3600,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        },
        SysCap {
            key: "custom.remove".to_owned(),
            category: "custom".to_owned(),
            name: "remove".to_owned(),
            display_name: "Remove This".to_owned(),
            present: true,
            version: None,
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 3600,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        },
    ];

    service.set_custom_syscap(node_id, custom_caps).unwrap();

    // Remove one key
    let result = service.remove_custom_syscap(node_id, vec!["custom.remove".to_owned()]);
    assert!(result.is_ok(), "Should successfully remove custom syscap");

    // Verify only one remains
    let syscap = service.get_node_syscap(node_id, false).unwrap();
    let has_keep = syscap.capabilities.iter().any(|c| c.key == "custom.keep");
    let has_remove = syscap.capabilities.iter().any(|c| c.key == "custom.remove");

    assert!(has_keep, "Should still have 'keep' capability");
    assert!(!has_remove, "Should not have 'remove' capability");
}

#[test]
fn test_clear_custom_syscap_removes_all_custom_capabilities() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    // Get initial syscap count (system capabilities)
    let initial_syscap = service.get_node_syscap(node_id, false).unwrap();
    let initial_count = initial_syscap.capabilities.len();

    // Set custom caps
    let custom_caps = vec![
        SysCap {
            key: "custom.one".to_owned(),
            category: "custom".to_owned(),
            name: "one".to_owned(),
            display_name: "One".to_owned(),
            present: true,
            version: None,
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 3600,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        },
        SysCap {
            key: "custom.two".to_owned(),
            category: "custom".to_owned(),
            name: "two".to_owned(),
            display_name: "Two".to_owned(),
            present: true,
            version: None,
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 3600,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        },
    ];

    service.set_custom_syscap(node_id, custom_caps).unwrap();

    // Verify custom caps were added
    let with_custom = service.get_node_syscap(node_id, false).unwrap();
    assert_eq!(
        with_custom.capabilities.len(),
        initial_count + 2,
        "Should have added 2 custom capabilities"
    );

    // Clear all custom caps
    let result = service.clear_custom_syscap(node_id);
    assert!(result.is_ok(), "Should successfully clear custom syscap");

    // Verify only system caps remain
    let after_clear = service.get_node_syscap(node_id, false).unwrap();
    assert_eq!(
        after_clear.capabilities.len(),
        initial_count,
        "Should only have system capabilities after clear"
    );

    let has_custom = after_clear
        .capabilities
        .iter()
        .any(|c| c.key.starts_with("custom."));
    assert!(!has_custom, "Should not have any custom capabilities");
}

#[test]
fn test_operations_with_empty_inputs() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    // All operations with empty inputs should succeed gracefully
    assert!(
        service.set_custom_syscap(node_id, vec![]).is_ok(),
        "Setting empty list should succeed"
    );
    assert!(
        service.remove_custom_syscap(node_id, vec![]).is_ok(),
        "Removing empty list should succeed"
    );
    assert!(
        service.clear_custom_syscap(node_id).is_ok(),
        "Clearing empty custom syscap should succeed"
    );
}

#[test]
fn test_service_default_trait() {
    let service = Service::default();
    let nodes = service.list_nodes();

    assert_eq!(
        nodes.len(),
        1,
        "Default service should initialize with current node"
    );
    assert!(
        !nodes[0].hostname.is_empty(),
        "Default service should have valid node"
    );
}
