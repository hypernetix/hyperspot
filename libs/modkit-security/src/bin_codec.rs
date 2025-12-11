use crate::SecurityCtx;
use bincode::config::{standard, Config};
use bincode::error::{DecodeError, EncodeError};
use thiserror::Error;

pub const SECCTX_BIN_VERSION: u8 = 1;

#[derive(Debug, Error)]
pub enum SecCtxEncodeError {
    #[error("security context serialization failed: {0:?}")]
    Bincode(EncodeError),
}

impl From<EncodeError> for SecCtxEncodeError {
    fn from(err: EncodeError) -> Self {
        SecCtxEncodeError::Bincode(err)
    }
}

#[derive(Debug, Error)]
pub enum SecCtxDecodeError {
    #[error("empty secctx blob")]
    Empty,

    #[error("unsupported secctx version: {0}")]
    UnsupportedVersion(u8),

    #[error("security context deserialization failed: {0:?}")]
    Bincode(DecodeError),
}

impl From<DecodeError> for SecCtxDecodeError {
    fn from(err: DecodeError) -> Self {
        SecCtxDecodeError::Bincode(err)
    }
}

const fn secctx_config() -> impl Config {
    standard().with_fixed_int_encoding().with_little_endian()
}

/// Encode SecurityCtx into a versioned binary blob.
/// This does not do any signing or encryption, it is just a transport format.
pub fn encode_bin(ctx: &SecurityCtx) -> Result<Vec<u8>, SecCtxEncodeError> {
    let mut buf = Vec::with_capacity(64);
    buf.push(SECCTX_BIN_VERSION);

    let cfg = secctx_config();
    let payload = bincode::serde::encode_to_vec(ctx, cfg)?;
    buf.extend_from_slice(&payload);

    Ok(buf)
}

/// Decode SecurityCtx from a versioned binary blob produced by encode_bin().
pub fn decode_bin(bytes: &[u8]) -> Result<SecurityCtx, SecCtxDecodeError> {
    if bytes.is_empty() {
        return Err(SecCtxDecodeError::Empty);
    }

    let version = bytes[0];
    if version != SECCTX_BIN_VERSION {
        return Err(SecCtxDecodeError::UnsupportedVersion(version));
    }

    let payload = &bytes[1..];
    let cfg = secctx_config();

    // decode_from_slice: Result<(T, usize), DecodeError>
    let (ctx, _len): (SecurityCtx, usize) = bincode::serde::decode_from_slice(payload, cfg)?;

    Ok(ctx)
}
