use crate::SecurityContext;
use postcard::Error as PostcardError;
use thiserror::Error;

pub const SECCTX_BIN_VERSION: u8 = 1;

#[derive(Debug, Error)]
pub enum SecCtxEncodeError {
    #[error("security context serialization failed: {0:?}")]
    Postcard(#[from] PostcardError),
}

#[derive(Debug, Error)]
pub enum SecCtxDecodeError {
    #[error("empty secctx blob")]
    Empty,

    #[error("unsupported secctx version: {0}")]
    UnsupportedVersion(u8),

    #[error("security context deserialization failed: {0:?}")]
    Postcard(#[from] PostcardError),
}

/// Encode `SecurityContext` into a versioned binary blob using `postcard`.
/// This does not do any signing or encryption, it is just a transport format.
///
/// # Errors
/// Returns `SecCtxEncodeError` if postcard serialization fails.
pub fn encode_bin(ctx: &SecurityContext) -> Result<Vec<u8>, SecCtxEncodeError> {
    let mut buf = Vec::with_capacity(64);
    buf.push(SECCTX_BIN_VERSION);

    let payload = postcard::to_allocvec(ctx)?;
    buf.extend_from_slice(&payload);

    Ok(buf)
}

/// Decode `SecurityContext` from a versioned binary blob produced by `encode_bin()`.
///
/// # Errors
/// Returns `SecCtxDecodeError::Empty` if the input is empty.
/// Returns `SecCtxDecodeError::UnsupportedVersion` if the version byte is not supported.
/// Returns `SecCtxDecodeError::Postcard` if postcard deserialization fails.
pub fn decode_bin(bytes: &[u8]) -> Result<SecurityContext, SecCtxDecodeError> {
    if bytes.is_empty() {
        return Err(SecCtxDecodeError::Empty);
    }

    let version = bytes[0];
    if version != SECCTX_BIN_VERSION {
        return Err(SecCtxDecodeError::UnsupportedVersion(version));
    }

    let payload = &bytes[1..];

    let ctx: SecurityContext = postcard::from_bytes(payload)?;

    Ok(ctx)
}
