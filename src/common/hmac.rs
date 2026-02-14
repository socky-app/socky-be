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

use hmac::digest::Output;
use hmac::{digest::KeyInit, Hmac, Mac};
use sha2::{Sha256, Sha512};

pub use hmac::digest::InvalidLength;

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

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    // Note: You may want to add `hex = "0.4"` to your dev-dependencies 
    // to easily compare byte arrays with standard hex strings.
    // Use `cargo add --dev hex`

    // --- SHA-256 Tests ---

    #[test]
    fn test_sha256_round_trip() {
        let key = b"super-secret-key";
        let msg = b"hello world";

        // 1. Generate tag
        let tag = hash_sha256(msg, key).expect("HMAC generation failed");

        // 2. Verify successfully
        assert!(verify_sha256(msg, key, &tag), "Verification should pass with correct key/msg");
    }

    #[test]
    fn test_sha256_known_answer_rfc4231() {
        // RFC 4231 Test Case 1
        // Key: val 0x0b (20 bytes)
        // Data: "Hi There"
        let key = [0x0b; 20];
        let msg = b"Hi There";
        let expected_hex = "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7";

        let tag = hash_sha256(msg, &key).unwrap();
        
        // Use hex crate if available, otherwise manual comparison
        assert_eq!(hex::encode(tag), expected_hex);
    }

    #[test]
    fn test_sha256_verification_failures() {
        let key = b"secret";
        let msg = b"data";
        let tag = hash_sha256(msg, key).unwrap();

        // 1. Test Wrong Message
        assert!(!verify_sha256(b"data modified", key, &tag), "Should fail on modified message");

        // 2. Test Wrong Key
        assert!(!verify_sha256(msg, b"wrong secret", &tag), "Should fail on wrong key");

        // 3. Test Corrupted Tag
        let mut corrupted_tag = tag;
        corrupted_tag[0] ^= 0xFF; // Flip bits in the first byte
        assert!(!verify_sha256(msg, key, &corrupted_tag), "Should fail on corrupted tag");
    }

    // --- SHA-512 Tests ---

    #[test]
    fn test_sha512_round_trip() {
        let key = b"another-secret";
        let msg = b"secure message";
        
        let tag = hash_sha512(msg, key).expect("HMAC generation failed");
        
        // Check output size is correct (64 bytes for SHA-512)
        assert_eq!(tag.len(), 64); 
        assert!(verify_sha512(msg, key, &tag));
    }

    #[test]
    fn test_sha512_known_answer_rfc4231() {
        // RFC 4231 Test Case 1 for HMAC-SHA-512
        let key = [0x0b; 20];
        let msg = b"Hi There";
        let expected_hex = "87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cdedaa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854";

        let tag = hash_sha512(msg, &key).unwrap();
        assert_eq!(hex::encode(tag), expected_hex);
    }

    // --- Edge Case Tests ---

    #[test]
    fn test_empty_inputs() {
        let key = b"key";
        let empty_msg = b"";
        
        // Empty message is valid
        let tag = hash_sha256(empty_msg, key).unwrap();
        assert!(verify_sha256(empty_msg, key, &tag));

        // Empty key (HMAC usually allows this, treating it as zero-padded or hashing it)
        let empty_key = b"";
        let tag2 = hash_sha256(b"msg", empty_key).unwrap();
        assert!(verify_sha256(b"msg", empty_key, &tag2));
    }

    #[test]
    fn test_key_handling() {
         // HMAC keys can be longer than the block size (they get hashed down)
         let long_key = [0u8; 1024]; 
         let msg = b"test";
         
         let result = hash_sha256(msg, &long_key);
         assert!(result.is_ok(), "Should accept long keys (HMAC standard behavior)");
    }
}