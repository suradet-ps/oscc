//! Opaque session tokens: 256 bits of randomness, stored as SHA-256.

use rand::RngCore;
use sha2::{Digest, Sha256};

/// Token entropy in bytes.
pub const TOKEN_BYTES: usize = 32;

/// Generates a token, returning `(raw_for_client, hash_for_storage)`.
///
/// The raw token leaves the server exactly once, in the login response;
/// only its hash is persisted, so a database dump cannot be replayed.
pub fn generate() -> (String, String) {
    let mut bytes = [0u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    let raw = hex::encode(bytes);
    let hash = token_hash(&raw);
    (raw, hash)
}

/// The storage hash of a raw token.
pub fn token_hash(raw: &str) -> String {
    hex::encode(Sha256::digest(raw.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tokens_are_hex_and_long_enough() {
        let (raw, hash) = generate();
        assert_eq!(raw.len(), TOKEN_BYTES * 2);
        assert_eq!(hash.len(), 64);
        assert!(raw.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generated_tokens_do_not_repeat() {
        let (first, _) = generate();
        let (second, _) = generate();
        assert_ne!(first, second);
    }

    #[test]
    fn hash_is_deterministic_and_not_the_raw_token() {
        let (raw, hash) = generate();
        assert_eq!(token_hash(&raw), hash);
        assert_ne!(hash, raw);
    }
}
