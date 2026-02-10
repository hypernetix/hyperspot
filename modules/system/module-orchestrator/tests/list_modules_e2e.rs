#![allow(clippy::unwrap_used, clippy::expect_used)]

//! End-to-end tests for the `GET /modules/v1/modules/active` REST endpoint.
//!
//! These tests build a real axum `Router` with the module orchestrator's routes
//! registered via `OperationBuilder`, then send HTTP requests using `tower::ServiceExt::oneshot`.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use modkit::registry::{ModuleDescriptor, ModuleRegistryCatalog};
use modkit::runtime::{Endpoint, ModuleInstance, ModuleManager};
use module_orchestrator::api::rest;
use std::collections::HashSet;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

use module_orchestrator::domain::service::ModulesService;

fn build_router(
    catalog: ModuleRegistryCatalog,
    manager: Arc<ModuleManager>,
    external_names: HashSet<String>,
) -> Router {
    let svc = Arc::new(ModulesService::new(
        Arc::new(catalog),
        manager,
        Arc::new(external_names),
    ));
    let openapi = api_gateway::ApiGateway::default();
    rest::routes::register_routes(Router::new(), &openapi, svc)
}

async fn get_modules(router: Router) -> (StatusCode, serde_json::Value) {
    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/modules/v1/modules/active")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

#[tokio::test]
async fn returns_200_with_empty_catalog() {
    let router = build_router(
        ModuleRegistryCatalog { modules: vec![] },
        Arc::new(ModuleManager::new()),
        HashSet::new(),
    );

    let (status, json) = get_modules(router).await;

    assert_eq!(status, StatusCode::OK);
    assert!(json.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn returns_compiled_in_modules_with_capabilities() {
    let catalog = ModuleRegistryCatalog {
        modules: vec![
            ModuleDescriptor {
                name: "api_gateway".to_owned(),
                deps: vec!["grpc_hub".to_owned()],
                capability_labels: vec!["rest".to_owned(), "system".to_owned()],
            },
            ModuleDescriptor {
                name: "grpc_hub".to_owned(),
                deps: vec![],
                capability_labels: vec!["grpc_hub".to_owned()],
            },
        ],
    };
    let router = build_router(catalog, Arc::new(ModuleManager::new()), HashSet::new());

    let (status, json) = get_modules(router).await;

    assert_eq!(status, StatusCode::OK);
    let modules = json.as_array().unwrap();
    assert_eq!(modules.len(), 2);

    // Sorted by name
    assert_eq!(modules[0]["name"], "api_gateway");
    assert_eq!(modules[0]["deployment_mode"], "compiled_in");
    assert_eq!(
        modules[0]["capabilities"],
        serde_json::json!(["rest", "system"])
    );
    assert_eq!(
        modules[0]["dependencies"],
        serde_json::json!(["grpc_hub"])
    );

    assert_eq!(modules[1]["name"], "grpc_hub");
    assert_eq!(modules[1]["deployment_mode"], "compiled_in");
}

#[tokio::test]
async fn returns_external_modules_as_out_of_process() {
    let router = build_router(
        ModuleRegistryCatalog { modules: vec![] },
        Arc::new(ModuleManager::new()),
        HashSet::from(["calculator".to_owned()]),
    );

    let (status, json) = get_modules(router).await;

    assert_eq!(status, StatusCode::OK);
    let modules = json.as_array().unwrap();
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0]["name"], "calculator");
    assert_eq!(modules[0]["deployment_mode"], "out_of_process");
    assert!(modules[0]["instances"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn includes_running_instances_with_grpc_services() {
    let catalog = ModuleRegistryCatalog {
        modules: vec![ModuleDescriptor {
            name: "my_module".to_owned(),
            deps: vec![],
            capability_labels: vec!["grpc".to_owned()],
        }],
    };
    let manager = Arc::new(ModuleManager::new());
    let instance_id = Uuid::new_v4();
    let instance = Arc::new(
        ModuleInstance::new("my_module", instance_id)
            .with_version("1.2.3")
            .with_grpc_service("my.Service", Endpoint::http("127.0.0.1", 9000)),
    );
    manager.register_instance(instance);

    let router = build_router(catalog, manager, HashSet::new());

    let (status, json) = get_modules(router).await;

    assert_eq!(status, StatusCode::OK);
    let module = &json.as_array().unwrap()[0];
    assert_eq!(module["name"], "my_module");
    // Module-level version derived from first instance
    assert_eq!(module["version"], "1.2.3");

    let instances = module["instances"].as_array().unwrap();
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0]["instance_id"], instance_id.to_string());
    assert_eq!(instances[0]["version"], "1.2.3");
    assert_eq!(instances[0]["state"], "registered");
    assert!(instances[0]["grpc_services"]["my.Service"]
        .as_str()
        .unwrap()
        .contains("127.0.0.1"));
}

#[tokio::test]
async fn compiled_in_module_overridden_to_external() {
    let catalog = ModuleRegistryCatalog {
        modules: vec![ModuleDescriptor {
            name: "calculator".to_owned(),
            deps: vec![],
            capability_labels: vec!["grpc".to_owned()],
        }],
    };
    let router = build_router(
        catalog,
        Arc::new(ModuleManager::new()),
        HashSet::from(["calculator".to_owned()]),
    );

    let (status, json) = get_modules(router).await;

    assert_eq!(status, StatusCode::OK);
    let module = &json.as_array().unwrap()[0];
    assert_eq!(module["name"], "calculator");
    assert_eq!(module["deployment_mode"], "out_of_process");
    // Still retains capabilities from catalog
    assert_eq!(module["capabilities"], serde_json::json!(["grpc"]));
}

#[tokio::test]
async fn dynamic_instances_without_catalog_entry() {
    let manager = Arc::new(ModuleManager::new());
    let instance = Arc::new(
        ModuleInstance::new("dynamic_svc", Uuid::new_v4()).with_version("0.5.0"),
    );
    manager.register_instance(instance);

    let router = build_router(
        ModuleRegistryCatalog { modules: vec![] },
        manager,
        HashSet::new(),
    );

    let (status, json) = get_modules(router).await;

    assert_eq!(status, StatusCode::OK);
    let module = &json.as_array().unwrap()[0];
    assert_eq!(module["name"], "dynamic_svc");
    assert_eq!(module["deployment_mode"], "out_of_process");
    assert_eq!(module["version"], "0.5.0");
    assert!(module["capabilities"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn plugins_field_omitted_when_empty() {
    let catalog = ModuleRegistryCatalog {
        modules: vec![ModuleDescriptor {
            name: "test".to_owned(),
            deps: vec![],
            capability_labels: vec![],
        }],
    };
    let router = build_router(catalog, Arc::new(ModuleManager::new()), HashSet::new());

    let (status, json) = get_modules(router).await;

    assert_eq!(status, StatusCode::OK);
    let module = &json.as_array().unwrap()[0];
    // plugins field should be absent (skip_serializing_if = Vec::is_empty)
    assert!(module.get("plugins").is_none());
}

#[tokio::test]
async fn version_omitted_when_no_instances() {
    let catalog = ModuleRegistryCatalog {
        modules: vec![ModuleDescriptor {
            name: "no_instances".to_owned(),
            deps: vec![],
            capability_labels: vec![],
        }],
    };
    let router = build_router(catalog, Arc::new(ModuleManager::new()), HashSet::new());

    let (status, json) = get_modules(router).await;

    assert_eq!(status, StatusCode::OK);
    let module = &json.as_array().unwrap()[0];
    // version should be absent when no instances report one
    assert!(module.get("version").is_none());
}

#[tokio::test]
async fn modules_are_sorted_alphabetically() {
    let catalog = ModuleRegistryCatalog {
        modules: vec![
            ModuleDescriptor {
                name: "zebra".to_owned(),
                deps: vec![],
                capability_labels: vec![],
            },
            ModuleDescriptor {
                name: "alpha".to_owned(),
                deps: vec![],
                capability_labels: vec![],
            },
            ModuleDescriptor {
                name: "middle".to_owned(),
                deps: vec![],
                capability_labels: vec![],
            },
        ],
    };
    let router = build_router(catalog, Arc::new(ModuleManager::new()), HashSet::new());

    let (status, json) = get_modules(router).await;

    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["alpha", "middle", "zebra"]);
}
