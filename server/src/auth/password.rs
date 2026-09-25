//! Argon2id password hashing.

use std::sync::OnceLock;

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

/// Hashes a password into a PHC string.
///
/// # Errors
///
/// Returns the hashing error when the parameters or RNG fail.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

/// Verifies a password against a stored PHC string. A malformed stored
/// hash verifies as `false` rather than erroring (fail closed).
pub fn verify_password(password: &str, stored: &str) -> bool {
    match PasswordHash::new(stored) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Burns the same Argon2 cost as a real verification when the username is
/// unknown, so response timing does not reveal which accounts exist.
pub fn dummy_verify(password: &str) {
    static DUMMY_HASH: OnceLock<String> = OnceLock::new();
    let hash = DUMMY_HASH.get_or_init(|| {
        hash_password("oscc-timing-equalizer").unwrap_or_else(|_| "not-a-phc-string".to_string())
    });
    let _ = verify_password(password, hash);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_round_trips() {
        let hash = hash_password("correct horse battery staple").expect("hash");
        assert!(verify_password("correct horse battery staple", &hash));
    }

    #[test]
    fn wrong_password_fails() {
        let hash = hash_password("correct horse battery staple").expect("hash");
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn malformed_hash_fails_closed() {
        assert!(!verify_password("anything", "not-a-phc-string"));
    }

    #[test]
    fn hashes_are_salted_per_call() {
        let first = hash_password("same-password").expect("hash");
        let second = hash_password("same-password").expect("hash");
        assert_ne!(first, second);
    }

    #[test]
    fn dummy_verify_never_panics() {
        dummy_verify("anything");
        dummy_verify("");
    }
}
