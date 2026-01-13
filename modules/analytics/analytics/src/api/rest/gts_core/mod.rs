// @fdd-change:fdd-analytics-feature-gts-core-change-platform-integration-fix
pub mod dto;
pub mod error_handler;
pub mod handlers;
pub mod routes;

pub use dto::{GtsEntityDto, GtsEntityListDto, GtsEntityRequestDto};
pub use error_handler::{GtsCoreError, ProblemDetails};
pub use routes::register_routes;
