use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use magic_crypt::MagicCryptTrait;
use std::path::Path;

#[derive(Clone)]
pub struct CredentialEncryption {
    key: Vec<u8>,
}

impl CredentialEncryption {
    pub fn new(data_dir: &Path) -> Self {
        let key_path = data_dir.join(".encryption_key");
        let key = if key_path.exists() {
            let b64 = std::fs::read_to_string(&key_path).unwrap_or_else(|_| generate_key_b64());
            base64_decode(&b64).unwrap_or_else(generate_key)
        } else {
            let key = generate_key();
            let b64 = base64::engine::general_purpose::STANDARD.encode(&key);
            let _ = std::fs::write(&key_path, &b64);
            key
        };

        Self { key }
    }

    pub fn encrypt(&self, plaintext: &str) -> String {
        let cipher = Aes256Gcm::new_from_slice(&self.key).expect("invalid key length");
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .expect("encryption failed");

        let mut result = Vec::with_capacity(1 + nonce_bytes.len() + ciphertext.len());
        result.push(1); // version byte: 1 = AES-GCM
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        base64::engine::general_purpose::STANDARD.encode(&result)
    }

    pub fn decrypt(&self, ciphertext: &str) -> Option<String> {
        let data = base64_decode(ciphertext)?;
        if data.is_empty() {
            return None;
        }

        let version = data[0];

        if version == 1 {
            // AES-GCM v1
            if data.len() < 13 {
                return None;
            }
            let nonce = Nonce::from_slice(&data[1..13]);
            let cipher = Aes256Gcm::new_from_slice(&self.key).ok()?;
            let plaintext = cipher.decrypt(nonce, &data[13..]).ok()?;
            String::from_utf8(plaintext).ok()
        } else {
            // Legacy magic-crypt (no version byte, try legacy)
            let legacy = magic_crypt::new_magic_crypt!(
                base64::engine::general_purpose::STANDARD.encode(&self.key),
                256
            );
            legacy.decrypt_base64_to_string(ciphertext).ok()
        }
    }
}

fn generate_key() -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..32).map(|_| rng.random::<u8>()).collect()
}

fn generate_key_b64() -> String {
    base64::engine::general_purpose::STANDARD.encode(generate_key())
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::STANDARD.decode(s).ok()
}
