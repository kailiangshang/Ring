use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::{Result, RingError};

type HmacSha256 = Hmac<Sha256>;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_DERIVATION_ITERATIONS: u32 = 600_000;

pub struct CredentialService {
    key: [u8; 32],
}

impl CredentialService {
    pub fn new(key: [u8; 32]) -> Self {
        CredentialService { key }
    }

    pub fn derive_key_from_password(password: &str) -> [u8; 32] {
        Self::pbkdf2_hmac_sha256(password.as_bytes(), b"ring-default-salt")
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut salt = [0u8; SALT_LEN];
        getrandom::getrandom(&mut salt)
            .map_err(|e| RingError::Internal(format!("rng failed: {}", e)))?;

        let derived_key = Self::pbkdf2_hmac_sha256(&self.key, &salt);

        let mut nonce_bytes = [0u8; NONCE_LEN];
        getrandom::getrandom(&mut nonce_bytes)
            .map_err(|e| RingError::Internal(format!("rng failed: {}", e)))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&derived_key)
            .map_err(|e| RingError::Internal(format!("cipher init failed: {}", e)))?;
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| RingError::Internal(format!("encryption failed: {}", e)))?;

        let mut output = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&output))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String> {
        let bytes = BASE64
            .decode(encrypted)
            .map_err(|e| RingError::Internal(format!("base64 decode failed: {}", e)))?;

        if bytes.len() < SALT_LEN + NONCE_LEN {
            return Err(RingError::Internal("ciphertext too short".into()));
        }

        let salt = &bytes[..SALT_LEN];
        let nonce_bytes = &bytes[SALT_LEN..SALT_LEN + NONCE_LEN];
        let ciphertext = &bytes[SALT_LEN + NONCE_LEN..];

        let derived_key = Self::pbkdf2_hmac_sha256(&self.key, salt);
        let cipher = Aes256Gcm::new_from_slice(&derived_key)
            .map_err(|e| RingError::Internal(format!("cipher init failed: {}", e)))?;
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| RingError::Internal(format!("decryption failed: {}", e)))?;
        String::from_utf8(plaintext).map_err(|e| RingError::Internal(format!("utf8 failed: {}", e)))
    }

    fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8]) -> [u8; 32] {
        let mut mac = <HmacSha256 as Mac>::new_from_slice(password)
            .expect("HMAC accepts any key size");
        mac.update(salt);
        mac.update(&1u32.to_be_bytes());
        let u_init = mac.finalize().into_bytes();
        let mut result = [0u8; 32];
        result.copy_from_slice(&u_init);
        let mut u = u_init;

        for _ in 1..KEY_DERIVATION_ITERATIONS {
            let mut mac = <HmacSha256 as Mac>::new_from_slice(password)
                .expect("HMAC accepts any key size");
            mac.update(&u);
            u = mac.finalize().into_bytes();
            for i in 0..32 {
                result[i] ^= u[i];
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let svc = CredentialService::new(key);
        let plaintext = "hello ring secret";
        let encrypted = svc.encrypt(plaintext).unwrap();
        let decrypted = svc.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_ciphertexts_each_time() {
        let key = [42u8; 32];
        let svc = CredentialService::new(key);
        let plaintext = "same input";
        let enc1 = svc.encrypt(plaintext).unwrap();
        let enc2 = svc.encrypt(plaintext).unwrap();
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let key1 = CredentialService::derive_key_from_password("test-password");
        let key2 = CredentialService::derive_key_from_password("test-password");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let key1 = CredentialService::derive_key_from_password("password-a");
        let key2 = CredentialService::derive_key_from_password("password-b");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_decrypt_tampered_fails() {
        let key = [42u8; 32];
        let svc = CredentialService::new(key);
        let encrypted = svc.encrypt("secret").unwrap();
        let mut bytes = BASE64.decode(&encrypted).unwrap();
        if let Some(last) = bytes.last_mut() {
            *last = !*last;
        }
        let tampered = BASE64.encode(&bytes);
        assert!(svc.decrypt(&tampered).is_err());
    }

    #[test]
    fn test_random_nonce_not_static() {
        let key = [42u8; 32];
        let svc = CredentialService::new(key);
        let enc = svc.encrypt("test").unwrap();
        let bytes = BASE64.decode(&enc).unwrap();
        let nonce = &bytes[SALT_LEN..SALT_LEN + NONCE_LEN];
        assert_ne!(nonce, b"ring-ring-no");
    }

    #[test]
    fn test_output_has_salt_nonce_ciphertext_layout() {
        let key = [42u8; 32];
        let svc = CredentialService::new(key);
        let enc = svc.encrypt("test").unwrap();
        let bytes = BASE64.decode(&enc).unwrap();
        assert!(bytes.len() > SALT_LEN + NONCE_LEN);
    }
}
