#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod client;

pub const SECCTX_METADATA_KEY: &str = "x-secctx-bin";

use modkit_security::{decode_bin, encode_bin, SecurityCtx};
use tonic::metadata::{MetadataMap, MetadataValue};
use tonic::Status;

/// Encode `SecurityCtx` into gRPC metadata.
pub fn attach_secctx(meta: &mut MetadataMap, ctx: &SecurityCtx) -> Result<(), Status> {
    let encoded = encode_bin(ctx).map_err(|e| Status::internal(format!("secctx encode: {e}")))?;

    meta.insert_bin(SECCTX_METADATA_KEY, MetadataValue::from_bytes(&encoded));
    Ok(())
}

/// Decode `SecurityCtx` from gRPC metadata.
pub fn extract_secctx(meta: &MetadataMap) -> Result<SecurityCtx, Status> {
    let raw = meta
        .get_bin(SECCTX_METADATA_KEY)
        .ok_or_else(|| Status::unauthenticated("missing secctx metadata"))?;

    let bytes = raw
        .to_bytes()
        .map_err(|e| Status::unauthenticated(format!("invalid secctx metadata: {e}")))?;

    decode_bin(bytes.as_ref()).map_err(|e| Status::unauthenticated(format!("secctx decode: {e}")))
}
