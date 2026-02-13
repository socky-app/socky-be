use hmac::digest::{InvalidLength, Output};
use hmac::{digest::KeyInit, Hmac, Mac};
use sha2::{Sha256, Sha512};

// Type Aliases for convenience
type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

// -----------------------------------------------------------------------------
// Internal Generic Helpers (Private)
// -----------------------------------------------------------------------------

fn hash<M>(value: &[u8], secret: &[u8]) -> Result<Output<M>, InvalidLength>
where
    M: Mac + KeyInit,
{
    let mut mac = <M as Mac>::new_from_slice(secret)?;
    mac.update(value);
    Ok(mac.finalize().into_bytes())
}

fn verify<M>(value: &[u8], secret: &[u8], expected: &[u8]) -> bool
where
    M: Mac + KeyInit,
{
    // If the key is invalid, we return false (verification failed)
    // rather than panicking or bubbling the error.
    let mut mac = match <M as Mac>::new_from_slice(secret) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(value);

    // verify_slice handles length checks and is constant-time
    mac.verify_slice(expected).is_ok()
}

// -----------------------------------------------------------------------------
// Public Concrete API
// -----------------------------------------------------------------------------

pub fn hash_sha256(value: &[u8], secret: &[u8]) -> Result<[u8; 32], InvalidLength> {
    let result = hash::<HmacSha256>(value, secret)?;
    Ok(result.into())
}

pub fn verify_sha256(value: &[u8], secret: &[u8], expected: &[u8]) -> bool {
    verify::<HmacSha256>(value, secret, expected)
}

pub fn hash_sha512(value: &[u8], secret: &[u8]) -> Result<[u8; 64], InvalidLength> {
    let result = hash::<HmacSha512>(value, secret)?;
    Ok(result.into())
}

pub fn verify_sha512(value: &[u8], secret: &[u8], expected: &[u8]) -> bool {
    verify::<HmacSha512>(value, secret, expected)
}
