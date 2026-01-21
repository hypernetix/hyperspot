#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Unit tests for the nodes registry storage layer
//!
//! These tests verify storage operations, concurrency, and edge cases.

use nodes_registry::domain::node_storage::NodeStorage;
use nodes_registry::{Node, SysCap};
use std::sync::Arc;
use std::thread;
use uuid::Uuid;

#[test]
fn test_storage_upsert_and_get_node() {
    let storage = NodeStorage::new();
    let node_id = Uuid::new_v4();

    let node = Node {
        id: node_id,
        hostname: "test-node".to_owned(),
        ip_address: Some("192.168.1.100".to_owned()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    storage.upsert_node(node);

    let retrieved = storage.get_node(node_id).unwrap();
    assert_eq!(retrieved.id, node_id);
    assert_eq!(retrieved.hostname, "test-node");
    assert_eq!(retrieved.ip_address, Some("192.168.1.100".to_owned()));
}

#[test]
fn test_storage_upsert_updates_existing_node() {
    let storage = NodeStorage::new();
    let node_id = Uuid::new_v4();

    // First upsert
    let node1 = Node {
        id: node_id,
        hostname: "initial-hostname".to_owned(),
        ip_address: Some("192.168.1.1".to_owned()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    storage.upsert_node(node1.clone());

    // Second upsert with same ID but different data
    let node2 = Node {
        id: node_id,
        hostname: "updated-hostname".to_owned(),
        ip_address: Some("192.168.1.2".to_owned()),
        created_at: node1.created_at,
        updated_at: chrono::Utc::now(),
    };
    storage.upsert_node(node2);

    // Should have the updated version
    let retrieved = storage.get_node(node_id).unwrap();
    assert_eq!(retrieved.hostname, "updated-hostname");
    assert_eq!(retrieved.ip_address, Some("192.168.1.2".to_owned()));
}

#[test]
fn test_storage_list_nodes() {
    let storage = NodeStorage::new();

    // Add multiple nodes
    for i in 0..3 {
        let node = Node {
            id: Uuid::new_v4(),
            hostname: format!("node-{i}"),
            ip_address: Some(format!("192.168.1.{i}")),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        storage.upsert_node(node);
    }

    let nodes = storage.list_nodes();
    assert_eq!(nodes.len(), 3);
}

#[test]
fn test_storage_operations_on_nonexistent_node() {
    let storage = NodeStorage::new();
    let fake_id = Uuid::new_v4();

    // All operations on nonexistent nodes should fail gracefully
    assert!(storage.get_node(fake_id).is_none());
    assert!(storage.get_sysinfo(fake_id).is_none());
    assert!(storage.get_syscap(fake_id).is_none());
    assert!(!storage.set_custom_syscap(fake_id, vec![]));
    assert!(!storage.remove_custom_syscap(fake_id, vec![]));
    assert!(!storage.clear_custom_syscap(fake_id));
}

#[test]
fn test_storage_concurrent_reads() {
    let storage: Arc<NodeStorage> = Arc::new(NodeStorage::new());
    let node_id = Uuid::new_v4();

    // Add a node
    let node = Node {
        id: node_id,
        hostname: "test".to_owned(),
        ip_address: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    storage.upsert_node(node);

    // Spawn multiple reader threads
    let mut handles = vec![];
    for _ in 0..10 {
        let storage_clone: Arc<NodeStorage> = Arc::clone(&storage);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let _ = storage_clone.get_node(node_id);
                let _ = storage_clone.list_nodes();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Node should still be accessible
    let result = storage.get_node(node_id);
    assert!(
        result.is_some(),
        "Node should still exist after concurrent reads"
    );
}

#[test]
fn test_storage_remove_multiple_custom_syscap() {
    let storage = NodeStorage::new();
    let node_id = Uuid::new_v4();

    let node = Node {
        id: node_id,
        hostname: "test".to_owned(),
        ip_address: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    storage.upsert_node(node);

    // Add multiple custom capabilities
    let caps = vec![
        SysCap {
            key: "custom.test1".to_owned(),
            category: "custom".to_owned(),
            name: "test1".to_owned(),
            display_name: "Test 1".to_owned(),
            present: true,
            version: None,
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 3600,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        },
        SysCap {
            key: "custom.test2".to_owned(),
            category: "custom".to_owned(),
            name: "test2".to_owned(),
            display_name: "Test 2".to_owned(),
            present: true,
            version: None,
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 3600,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        },
        SysCap {
            key: "custom.test3".to_owned(),
            category: "custom".to_owned(),
            name: "test3".to_owned(),
            display_name: "Test 3".to_owned(),
            present: true,
            version: None,
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 3600,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        },
    ];
    storage.set_custom_syscap(node_id, caps);

    // Remove multiple at once
    storage.remove_custom_syscap(
        node_id,
        vec!["custom.test1".to_owned(), "custom.test3".to_owned()],
    );

    let syscap = storage.get_syscap(node_id).unwrap();

    // test1 and test3 should be removed
    assert!(
        !syscap.capabilities.iter().any(|c| c.key == "custom.test1"),
        "test1 should be removed"
    );
    assert!(
        !syscap.capabilities.iter().any(|c| c.key == "custom.test3"),
        "test3 should be removed"
    );

    // test2 should remain
    assert!(
        syscap.capabilities.iter().any(|c| c.key == "custom.test2"),
        "test2 should remain"
    );
}

#[test]
fn test_storage_clear_custom_syscap_preserves_system() {
    let storage = NodeStorage::new();
    let node_id = Uuid::new_v4();

    let node = Node {
        id: node_id,
        hostname: "test".to_owned(),
        ip_address: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    storage.upsert_node(node);

    // Add system syscap
    let system_syscap = nodes_registry::NodeSysCap {
        node_id,
        capabilities: vec![SysCap {
            key: "system.test".to_owned(),
            category: "system".to_owned(),
            name: "test".to_owned(),
            display_name: "System Test".to_owned(),
            present: true,
            version: None,
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 3600,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        }],
        collected_at: chrono::Utc::now(),
    };
    storage.update_syscap_system(node_id, system_syscap);

    // Add custom syscap
    let custom_caps = vec![SysCap {
        key: "custom.test".to_owned(),
        category: "custom".to_owned(),
        name: "test".to_owned(),
        display_name: "Custom Test".to_owned(),
        present: true,
        version: None,
        amount: None,
        amount_dimension: None,
        details: None,
        cache_ttl_secs: 3600,
        fetched_at_secs: chrono::Utc::now().timestamp(),
    }];
    storage.set_custom_syscap(node_id, custom_caps);

    // Clear custom syscap
    storage.clear_custom_syscap(node_id);

    let syscap = storage.get_syscap(node_id).unwrap();

    // System syscap should remain
    assert!(
        syscap.capabilities.iter().any(|c| c.key == "system.test"),
        "System capability should remain"
    );

    // Custom syscap should be gone
    assert!(
        !syscap.capabilities.iter().any(|c| c.key == "custom.test"),
        "Custom capability should be removed"
    );
}
