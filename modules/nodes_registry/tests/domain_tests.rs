#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for the nodes registry domain layer
//!
//! These tests verify domain behavior including caching, persistence, and edge cases.

use nodes_registry::contract::SysCap;
use nodes_registry::domain::service::Service;

#[test]
fn test_custom_syscap_persists_across_syscap_refreshes() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    // Set custom capability
    let custom_cap = vec![SysCap {
        key: "custom.persistent".to_owned(),
        category: "custom".to_owned(),
        name: "persistent".to_owned(),
        display_name: "Persistent Feature".to_owned(),
        present: true,
        version: Some("1.0.0".to_owned()),
        amount: None,
        amount_dimension: None,
        details: None,
        cache_ttl_secs: 3600,
        fetched_at_secs: chrono::Utc::now().timestamp(),
    }];

    service.set_custom_syscap(node_id, custom_cap).unwrap();

    // Force refresh of syscap
    let syscap_before = service.get_node_syscap(node_id, true).unwrap();

    // Custom capability should still be present
    let has_custom = syscap_before
        .capabilities
        .iter()
        .any(|c| c.key == "custom.persistent");
    assert!(has_custom, "Custom capability should persist after refresh");

    // Refresh again
    let syscap_after = service.get_node_syscap(node_id, true).unwrap();
    let has_custom = syscap_after
        .capabilities
        .iter()
        .any(|c| c.key == "custom.persistent");
    assert!(
        has_custom,
        "Custom capability should still persist after second refresh"
    );
}

#[test]
fn test_sysinfo_collection_is_idempotent() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    // Collect sysinfo multiple times
    let result1 = service.get_node_sysinfo(node_id);
    let result2 = service.get_node_sysinfo(node_id);
    let result3 = service.get_node_sysinfo(node_id);

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    let sysinfo1 = result1.unwrap();
    let sysinfo2 = result2.unwrap();
    let sysinfo3 = result3.unwrap();

    // All should return the same cached data
    assert_eq!(sysinfo1.node_id, sysinfo2.node_id);
    assert_eq!(sysinfo2.node_id, sysinfo3.node_id);
    assert_eq!(sysinfo1.collected_at, sysinfo2.collected_at);
    assert_eq!(sysinfo2.collected_at, sysinfo3.collected_at);
}

#[test]
fn test_removing_nonexistent_custom_syscap_keys_succeeds() {
    let service = Service::new();
    let nodes = service.list_nodes();
    let node_id = nodes[0].id;

    // First add some custom capabilities
    let custom_caps = vec![
        SysCap {
            key: "custom.existing1".to_owned(),
            category: "custom".to_owned(),
            name: "existing1".to_owned(),
            display_name: "Existing 1".to_owned(),
            present: true,
            version: Some("1.0.0".to_owned()),
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 3600,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        },
        SysCap {
            key: "custom.existing2".to_owned(),
            category: "custom".to_owned(),
            name: "existing2".to_owned(),
            display_name: "Existing 2".to_owned(),
            present: true,
            version: Some("2.0.0".to_owned()),
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 3600,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        },
    ];

    service.set_custom_syscap(node_id, custom_caps).unwrap();

    // Try to remove keys that don't exist
    let result = service.remove_custom_syscap(
        node_id,
        vec![
            "custom.nonexistent1".to_owned(),
            "custom.nonexistent2".to_owned(),
        ],
    );

    assert!(
        result.is_ok(),
        "Removing nonexistent keys should succeed (no-op)"
    );

    // Verify that existing capabilities are still present
    let syscap = service.get_node_syscap(node_id, false).unwrap();
    let has_existing1 = syscap
        .capabilities
        .iter()
        .any(|c| c.key == "custom.existing1");
    let has_existing2 = syscap
        .capabilities
        .iter()
        .any(|c| c.key == "custom.existing2");

    assert!(
        has_existing1,
        "Existing capability 'custom.existing1' should not be removed"
    );
    assert!(
        has_existing2,
        "Existing capability 'custom.existing2' should not be removed"
    );
}
