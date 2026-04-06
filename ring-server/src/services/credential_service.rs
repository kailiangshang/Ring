use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use crate::error::{Result, RingError};

pub struct CredentialService {
    key: [u8; 32],
}

impl CredentialService {
    pub fn new(key: [u8; 32]) -> Self {
        CredentialService { key }
    }

    pub fn derive_key_from_password(password: &str) -> [u8; 32] {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        password.hash(&mut hasher);
        let hash = hasher.finish();
        let mut key = [0u8; 32];
        for i in 0..4 {
            let offset = i * 8;
            if offset + 8 <= 32 {
                key[offset..offset + 8].copy_from_slice(&hash.to_le_bytes());
            }
        }
        key
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| RingError::Internal(format!("cipher init failed: {}", e)))?;
        let nonce = Nonce::from_slice(b"ring-ring-no");
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| RingError::Internal(format!("encryption failed: {}", e)))?;
        Ok(BASE64.encode(&ciphertext))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| RingError::Internal(format!("cipher init failed: {}", e)))?;
        let bytes = BASE64
            .decode(encrypted)
            .map_err(|e| RingError::Internal(format!("base64 decode failed: {}", e)))?;
        let nonce = Nonce::from_slice(b"ring-ring-no");
        let plaintext = cipher
            .decrypt(nonce, bytes.as_ref())
            .map_err(|e| RingError::Internal(format!("decryption failed: {}", e)))?;
        String::from_utf8(plaintext).map_err(|e| RingError::Internal(format!("utf8 failed: {}", e)))
    }
}
