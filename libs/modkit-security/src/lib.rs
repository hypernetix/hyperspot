#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
pub mod bin_codec;
pub mod constants;
pub mod context;
pub mod prelude;
pub mod scope;
pub mod subject;

pub use constants::{ROOT_SUBJECT_ID, ROOT_TENANT_ID};
pub use context::{PolicyEngine, SecurityContext, SimplePolicyEngine};
pub use scope::AccessScope;
pub use subject::Subject;

pub use bin_codec::{
    decode_bin, encode_bin, SecCtxDecodeError, SecCtxEncodeError, SECCTX_BIN_VERSION,
};
