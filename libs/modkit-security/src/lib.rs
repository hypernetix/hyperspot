#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, warnings)]

pub mod bin_codec;
pub mod constants;
pub mod prelude;
pub mod scope;
pub mod security_ctx;
pub mod subject;

pub use constants::{ROOT_SUBJECT_ID, ROOT_TENANT_ID};
pub use scope::AccessScope;
pub use security_ctx::SecurityCtx;
pub use subject::Subject;

pub use bin_codec::{
    decode_bin, encode_bin, SecCtxDecodeError, SecCtxEncodeError, SECCTX_BIN_VERSION,
};
