// TODO: DE0301 - refactor to remove modkit_db dependency from domain layer
// This module currently uses modkit_db types which violates DDD
#![allow(unknown_lints)]
#![allow(de0301_no_infra_in_domain)]

pub mod error;
pub mod events;
pub mod local_client;
pub mod ports;
pub mod repos;
pub mod service;
