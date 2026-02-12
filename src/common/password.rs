use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha512;
use thiserror::Error;

type HmacAlgorithm = Hmac<Sha512>;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Password hashing failed")]
    PasswordHashingFailed,
}

/// Password utilities for secure hashing and verification.
pub struct PasswordUtils;

impl PasswordUtils {
    /// Hashes a plain-text password using Argon2.
    ///
    /// This function generates a random salt and uses Argon2 with default parameters
    /// to create a secure hash of the provided password.
    /// 
    /// Consider calling this function using `spawn_blocking`, to avoid blocking an 
    /// executor thread with the hashing operation.
    ///
    /// # Arguments
    ///
    /// * `password` - The plain-text password to hash
    /// * `pepper` - The pepper to use in the hashing process
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The hashed password as a string
    /// * `Err(Error::PasswordHashingFailed)` - If hashing fails
    pub fn hash_password(password: &str, pepper: &str) -> Result<String, PasswordError> {
        let peppered_password = Self::compute_peppered_password(password, pepper)?;
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(&peppered_password, &salt)
            .map_err(|_| PasswordError::PasswordHashingFailed)?
            .to_string();

        Ok(password_hash)
    }

    /// Verifies a password against a hash.
    ///
    /// This function parses the stored hash and verifies if the provided
    /// plain-text password matches the hash.
    /// 
    /// Consider calling this function using `spawn_blocking`, to avoid blocking an 
    /// executor thread with the hashing operation.
    ///
    /// # Arguments
    ///
    /// * `password` - The plain-text password to verify
    /// * `hash` - The stored hash to verify against
    /// * `pepper` - The pepper to use in the hashing process
    ///
    /// # Returns
    ///
    /// * `true` - If the password matches the hash
    /// * `false` - If the password doesn't match or hash parsing fails
    pub fn verify_password(password: &str, hash: &str, pepper: &str) -> bool {
        let peppered_password = match Self::compute_peppered_password(password, pepper) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(&peppered_password, &parsed_hash)
            .is_ok()
    }

    fn compute_peppered_password(password: &str, pepper: &str) -> Result<Vec<u8>, PasswordError> {
        let hmac = HmacAlgorithm::new_from_slice(pepper.as_bytes())
                .map_err(|_| PasswordError::PasswordHashingFailed)?;

        Ok(hmac
            .chain_update(password.as_bytes())
            .finalize()
            .into_bytes()
            .to_vec()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let pepper = "pepper";
        let password = "password";

        // Test hashing
        let hash = PasswordUtils::hash_password(password, pepper).expect("Should hash password");
        assert!(!hash.is_empty());

        // Test verification with correct password
        assert!(PasswordUtils::verify_password(password, &hash, pepper));

        // Test verification with incorrect password
        assert!(!PasswordUtils::verify_password("wrong_password", &hash, pepper));

        // Test verification with invalid hash
        assert!(!PasswordUtils::verify_password(password, "invalid_hash", pepper));
    }

    #[test]
    fn test_different_passwords_produce_different_hashes() {
        let pepper = "pepper";
        let password1 = "password1";
        let password2 = "password2";

        let hash1 = PasswordUtils::hash_password(password1, pepper).expect("Should hash password1");
        let hash2 = PasswordUtils::hash_password(password2, pepper).expect("Should hash password2");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_produces_different_hashes() {
        // Due to random salt, same password should produce different hashes
        let pepper = "pepper";
        let password = "password";

        let hash1 = PasswordUtils::hash_password(password, pepper).expect("Should hash password");
        let hash2 = PasswordUtils::hash_password(password, pepper).expect("Should hash password");

        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(PasswordUtils::verify_password(password, &hash1, pepper));
        assert!(PasswordUtils::verify_password(password, &hash2, pepper));
    }

    #[test]
    fn test_different_peppers_produce_different_hashes() {
        let pepper1 = "pepper1";
        let pepper2 = "pepper2";
        let password = "password";

        let hash1 = PasswordUtils::hash_password(password, pepper1).expect("Should hash password");
        let hash2 = PasswordUtils::hash_password(password, pepper2).expect("Should hash password");

        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(PasswordUtils::verify_password(password, &hash1, pepper1));
        assert!(PasswordUtils::verify_password(password, &hash2, pepper2));
    }
}
