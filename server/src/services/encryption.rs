use magic_crypt::MagicCryptTrait;
use std::path::Path;

#[derive(Clone)]
pub struct CredentialEncryption {
    key: String,
}

impl CredentialEncryption {
    pub fn new(data_dir: &Path) -> Self {
        let key_path = data_dir.join(".encryption_key");
        let key = if key_path.exists() {
            std::fs::read_to_string(&key_path).unwrap_or_else(|_| generate_key())
        } else {
            let key = generate_key();
            let _ = std::fs::write(&key_path, &key);
            key
        };

        Self { key }
    }

    pub fn encrypt(&self, plaintext: &str) -> String {
        let cipher = magic_crypt::new_magic_crypt!(&self.key, 256);
        cipher.encrypt_str_to_base64(plaintext)
    }

    pub fn decrypt(&self, ciphertext: &str) -> Option<String> {
        let cipher = magic_crypt::new_magic_crypt!(&self.key, 256);
        cipher.decrypt_base64_to_string(ciphertext).ok()
    }
}

fn generate_key() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.random::<u8>()).collect();
    base64::encode(&bytes)
}
