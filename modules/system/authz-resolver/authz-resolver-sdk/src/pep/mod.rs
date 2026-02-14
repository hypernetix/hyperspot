//! PEP (Policy Enforcement Point) helpers.
//!
//! - [`PolicyEnforcer`] — PEP object (build → evaluate → compile)
//! - [`ResourceType`] — Static descriptor for a resource type + its supported properties
//! - [`compile_to_access_scope`] — Low-level: compile evaluation response into `AccessScope`

pub mod compiler;
pub mod enforcer;

pub use compiler::{ConstraintCompileError, compile_to_access_scope};
pub use enforcer::{AccessRequest, EnforcerError, IntoPropertyValue, PolicyEnforcer, ResourceType};
