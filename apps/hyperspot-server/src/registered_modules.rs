// This file is used to ensure that all modules are linked and registered via inventory
// In future we can simply DX via build.rs which will collect all crates in ./modules and generate this file.
// But for now we will manually maintain this file.
#![allow(unused_imports)]

use api_ingress as _;
use directory_service as _;
use file_parser as _;
use grpc_hub as _;
#[cfg(feature = "users-info-example")]
use users_info as _;
