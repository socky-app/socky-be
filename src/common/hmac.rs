//! HMAC helpers using SHA-256 and SHA-512.
//!
//! This module provides small convenience wrappers around the `hmac` crate
//! to compute and verify HMAC tags with SHA-256 and SHA-512. The public API
//! exposes `hash_sha256`, `verify_sha256`, `hash_sha512`, and `verify_sha512`.
//!
//! # Examples
//!
//! ```rust
//! use socky_be::common::hmac;
//!
//! let secret = b"my secret";
//! let msg = b"important message";
//! let tag = hmac::hash_sha256(msg, secret).unwrap();
//! assert!(hmac::verify_sha256(msg, secret, &tag));
//! ```

use hmac::digest::{InvalidLength, Output};
use hmac::{digest::KeyInit, Hmac, Mac};
use sha2::{Sha256, Sha512};

/// Convenience alias for HMAC-SHA256 (`Hmac<Sha256>`).
type HmacSha256 = Hmac<Sha256>;

/// Convenience alias for HMAC-SHA512 (`Hmac<Sha512>`).
type HmacSha512 = Hmac<Sha512>;

/// Compute an HMAC-SHA256 tag for `value` using `secret`.
///
/// Returns a 32-byte array on success. Returns `InvalidLength` when the
/// provided `secret` is not an accepted key length for the underlying HMAC
/// implementation.
pub fn hash_sha256(value: &[u8], secret: &[u8]) -> Result<[u8; 32], InvalidLength> {
    let result = hash::<HmacSha256>(value, secret)?;
    Ok(result.into())
}

/// Verify that `expected` is the HMAC-SHA256 tag for `value` with `secret`.
///
/// Returns `true` when the tag matches, `false` otherwise. If the `secret`
/// cannot be used to initialize the HMAC instance, this function returns
/// `false`.
pub fn verify_sha256(value: &[u8], secret: &[u8], expected: &[u8]) -> bool {
    verify::<HmacSha256>(value, secret, expected)
}

/// Compute an HMAC-SHA512 tag for `value` using `secret`.
///
/// Returns a 64-byte array on success. Returns `InvalidLength` when the
/// provided `secret` is not an accepted key length for the underlying HMAC
/// implementation.
pub fn hash_sha512(value: &[u8], secret: &[u8]) -> Result<[u8; 64], InvalidLength> {
    let result = hash::<HmacSha512>(value, secret)?;
    Ok(result.into())
}

/// Verify that `expected` is the HMAC-SHA512 tag for `value` with `secret`.
///
/// Returns `true` when the tag matches, `false` otherwise. If the `secret`
/// cannot be used to initialize the HMAC instance, this function returns
/// `false`.
pub fn verify_sha512(value: &[u8], secret: &[u8], expected: &[u8]) -> bool {
    verify::<HmacSha512>(value, secret, expected)
}

/// Generic helper that computes an HMAC of type `M`.
///
/// This is a private helper used by the public `hash_*` functions.
fn hash<M>(value: &[u8], secret: &[u8]) -> Result<Output<M>, InvalidLength>
where
    M: Mac + KeyInit,
{
    let mut mac = <M as Mac>::new_from_slice(secret)?;
    mac.update(value);
    Ok(mac.finalize().into_bytes())
}

/// Generic helper that verifies an HMAC of type `M`.
///
/// Returns `true` when the computed tag matches `expected`.
fn verify<M>(value: &[u8], secret: &[u8], expected: &[u8]) -> bool
where
    M: Mac + KeyInit,
{
    let Ok(mut mac) = <M as Mac>::new_from_slice(secret) else {
        return false;
    };

    mac.update(value);
    mac.verify_slice(expected).is_ok()
}
