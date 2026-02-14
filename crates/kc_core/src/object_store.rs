use crate::app_error::{AppError, AppResult};
use crate::hashing::blake3_hex_prefixed;
use crate::types::ObjectHash;
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;

const ENCRYPTED_MAGIC: &[u8; 4] = b"KCE1";

#[derive(Debug, Clone)]
pub struct ObjectStoreEncryptionContext {
    pub key: [u8; 32],
    pub key_reference: String,
}

pub struct ObjectStore {
    pub objects_dir: PathBuf,
    pub encryption: Option<ObjectStoreEncryptionContext>,
}

pub fn derive_object_store_key(
    passphrase: &str,
    salt_id: &str,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> AppResult<[u8; 32]> {
    if salt_id.len() < 8 {
        return Err(AppError::new(
            "KC_ENCRYPTION_KEY_INVALID",
            "encryption",
            "salt_id must be at least 8 bytes",
            false,
            serde_json::json!({ "salt_id_len": salt_id.len() }),
        ));
    }
    let params = Params::new(memory_kib, iterations, parallelism, Some(32)).map_err(|e| {
        AppError::new(
            "KC_ENCRYPTION_KEY_INVALID",
            "encryption",
            "invalid argon2 parameters",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt_id.as_bytes(), &mut key)
        .map_err(|e| {
            AppError::new(
                "KC_ENCRYPTION_KEY_INVALID",
                "encryption",
                "failed deriving encryption key from passphrase",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    Ok(key)
}

impl ObjectStore {
    pub fn new(objects_dir: PathBuf) -> Self {
        Self {
            objects_dir,
            encryption: None,
        }
    }

    pub fn with_encryption(objects_dir: PathBuf, encryption: ObjectStoreEncryptionContext) -> Self {
        Self {
            objects_dir,
            encryption: Some(encryption),
        }
    }

    fn deterministic_nonce(
        object_hash: &ObjectHash,
        key_reference: &str,
    ) -> [u8; 24] {
        let material = format!("{}:{}", object_hash.0, key_reference);
        let digest = blake3::hash(material.as_bytes());
        let mut nonce = [0u8; 24];
        nonce.copy_from_slice(&digest.as_bytes()[0..24]);
        nonce
    }

    fn maybe_encrypt_bytes(&self, object_hash: &ObjectHash, bytes: &[u8]) -> AppResult<Vec<u8>> {
        let Some(enc) = &self.encryption else {
            return Ok(bytes.to_vec());
        };
        let nonce = Self::deterministic_nonce(object_hash, &enc.key_reference);
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&enc.key));
        let ciphertext = cipher
            .encrypt(XNonce::from_slice(&nonce), bytes)
            .map_err(|e| {
                AppError::new(
                    "KC_ENCRYPTION_KEY_INVALID",
                    "encryption",
                    "failed encrypting object payload",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        let mut out = Vec::with_capacity(ENCRYPTED_MAGIC.len() + nonce.len() + ciphertext.len());
        out.extend_from_slice(ENCRYPTED_MAGIC);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    fn maybe_decrypt_bytes(&self, object_hash: &ObjectHash, bytes: &[u8]) -> AppResult<Vec<u8>> {
        if !bytes.starts_with(ENCRYPTED_MAGIC) {
            return Ok(bytes.to_vec());
        }
        let enc = self.encryption.as_ref().ok_or_else(|| {
            AppError::new(
                "KC_ENCRYPTION_REQUIRED",
                "encryption",
                "encrypted object payload requires encryption context",
                false,
                serde_json::json!({ "object_hash": object_hash.0 }),
            )
        })?;
        if bytes.len() < ENCRYPTED_MAGIC.len() + 24 {
            return Err(AppError::new(
                "KC_ENCRYPTION_UNSUPPORTED",
                "encryption",
                "encrypted object payload has invalid format",
                false,
                serde_json::json!({ "object_hash": object_hash.0, "bytes": bytes.len() }),
            ));
        }
        let nonce = &bytes[ENCRYPTED_MAGIC.len()..ENCRYPTED_MAGIC.len() + 24];
        let ciphertext = &bytes[ENCRYPTED_MAGIC.len() + 24..];
        let expected_nonce = Self::deterministic_nonce(object_hash, &enc.key_reference);
        if nonce != expected_nonce {
            return Err(AppError::new(
                "KC_ENCRYPTION_KEY_INVALID",
                "encryption",
                "encryption key context does not match stored object payload",
                false,
                serde_json::json!({ "object_hash": object_hash.0 }),
            ));
        }
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&enc.key));
        cipher
            .decrypt(XNonce::from_slice(nonce), ciphertext)
            .map_err(|e| {
                AppError::new(
                    "KC_ENCRYPTION_KEY_INVALID",
                    "encryption",
                    "failed decrypting object payload",
                    false,
                    serde_json::json!({ "error": e.to_string(), "object_hash": object_hash.0 }),
                )
            })
    }

    fn file_path_for_hash(&self, object_hash: &ObjectHash) -> AppResult<PathBuf> {
        if !object_hash.0.starts_with("blake3:") || object_hash.0.len() < 9 {
            return Err(AppError::new(
                "KC_HASH_INVALID_FORMAT",
                "object_store",
                "invalid object hash format",
                false,
                serde_json::json!({ "object_hash": object_hash.0 }),
            ));
        }
        let digest = &object_hash.0[7..];
        let prefix = &digest[0..2];
        Ok(self.objects_dir.join(prefix).join(&object_hash.0))
    }

    pub fn put_bytes(
        &self,
        conn: &Connection,
        bytes: &[u8],
        created_event_id: i64,
    ) -> AppResult<ObjectHash> {
        let hash = ObjectHash(blake3_hex_prefixed(bytes));
        let path = self.file_path_for_hash(&hash)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::new(
                    "KC_INGEST_READ_FAILED",
                    "object_store",
                    "failed to create object hash prefix directory",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": parent }),
                )
            })?;
        }

        if !path.exists() {
            let stored_bytes = self.maybe_encrypt_bytes(&hash, bytes)?;
            fs::write(&path, &stored_bytes).map_err(|e| {
                AppError::new(
                    "KC_INGEST_READ_FAILED",
                    "object_store",
                    "failed to write object bytes",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": path }),
                )
            })?;
        }

        let relpath = format!("{}/{}", &hash.0[7..9], hash.0);
        conn.execute(
            "INSERT OR IGNORE INTO objects (object_hash, bytes, relpath, created_event_id) VALUES (?1, ?2, ?3, ?4)",
            params![hash.0, bytes.len() as i64, relpath, created_event_id],
        )
        .map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "object_store",
                "failed to insert object metadata",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        Ok(hash)
    }

    pub fn get_bytes(&self, object_hash: &ObjectHash) -> AppResult<Vec<u8>> {
        let path = self.file_path_for_hash(object_hash)?;
        let raw = fs::read(&path).map_err(|e| {
            AppError::new(
                "KC_INGEST_READ_FAILED",
                "object_store",
                "failed to read object bytes",
                false,
                serde_json::json!({ "error": e.to_string(), "path": path }),
            )
        })?;
        self.maybe_decrypt_bytes(object_hash, &raw)
    }

    pub fn exists(&self, object_hash: &ObjectHash) -> AppResult<bool> {
        Ok(self.file_path_for_hash(object_hash)?.exists())
    }
}
