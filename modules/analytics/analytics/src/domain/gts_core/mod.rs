// @fdd-change:fdd-analytics-feature-gts-core-change-routing-infrastructure
pub mod field_handler;
pub mod identifier;
pub mod query_validator;
pub mod router;
pub mod routing_table;

pub use field_handler::{FieldCategory, FieldHandler};
pub use identifier::GtsTypeIdentifier;
pub use query_validator::{QueryValidator, ValidationError};
pub use router::GtsCoreRouter;
pub use routing_table::{DomainHandler, RoutingTable};
