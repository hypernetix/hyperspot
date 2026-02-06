// This file is used to ensure that all modules are linked and registered via inventory
// In future we can simply DX via build.rs which will collect all crates in ./modules and generate this file.
// But for now we will manually maintain this file.
#![allow(unused_imports)]

use api_gateway as _;
use file_parser as _;
use grpc_hub as _;
use license_enforcer_gw as _;
use module_orchestrator as _;
use nodes_registry as _;
use simple_user_settings as _;
use tenant_resolver_gw as _;
use types as _;
use types_registry as _;

#[cfg(feature = "single-tenant")]
use single_tenant_tr_plugin as _;

#[cfg(feature = "static-tenants")]
use static_tr_plugin as _;

#[cfg(feature = "no-licensing-cache")]
use nocache_plugin as _;

#[cfg(feature = "inmemory-licensing-cache")]
use inmemory_cache_plugin as _;

#[cfg(feature = "static-licenses")]
use static_licenses_plugin as _;

// === Example Features ===

#[cfg(feature = "users-info-example")]
use users_info as _;

#[cfg(feature = "oop-example")]
use calculator_gateway as _;

#[cfg(feature = "oop-example")]
use calculator as _;

#[cfg(feature = "tenant-resolver-example")]
use contoso_tr_plugin as _;
#[cfg(feature = "tenant-resolver-example")]
use fabrikam_tr_plugin as _;
#[cfg(feature = "tenant-resolver-example")]
use tenant_resolver_example_gw as _;
