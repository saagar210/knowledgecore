# Encryption at Rest v1

## Purpose
Define active Phase M behavior for optional object-store encryption at rest with vault schema v2 metadata.

## Invariants
- Object hash remains `blake3(plaintext_bytes)` and remains Tier 1 deterministic.
- Encryption applies to object payload bytes on disk only in v1.
- SQLite file encryption is out of scope for v1.
- Vault schema v2 is required for active encryption metadata.
- UI routes only on `AppError.code`.

## Non-goals
- SQLite database encryption.
- Remote key management.
- Cross-device key escrow.

## Vault schema contract (v2)
- `schema_version: 2`
- `encryption.enabled: bool`
- `encryption.mode: "object_store_xchacha20poly1305"`
- `encryption.kdf.algorithm: "argon2id"`
- `encryption.kdf.memory_kib: u32`
- `encryption.kdf.iterations: u32`
- `encryption.kdf.parallelism: u32`
- `encryption.kdf.salt_id: String`
- `encryption.key_reference: Option<String>`

Legacy compatibility:
- `vault_open` accepts schema v1 and normalizes to v2 default encryption-disabled model in memory.

## Object-store encryption contract
- Cipher: XChaCha20Poly1305.
- Key derivation: Argon2id from passphrase and vault KDF parameters.
- Nonce derivation: deterministic from `object_hash` and `key_reference`.
- Stored blob framing:
  - bytes prefix magic: `KCE1`
  - nonce: 24 bytes
  - ciphertext: remaining bytes

Security note:
- Deterministic nonce derivation preserves dedupe determinism but leaks equality of plaintext payloads with identical key context.

## AppError codes (active)
- `KC_ENCRYPTION_KEY_INVALID`
- `KC_ENCRYPTION_UNSUPPORTED`
- `KC_ENCRYPTION_REQUIRED`
- `KC_ENCRYPTION_MIGRATION_FAILED`

## Acceptance tests
- v2 vault init writes `schema_version=2` and default encryption-disabled block.
- v1 vault opens and normalizes to v2 model.
- Encrypted object payload round-trip succeeds with correct key.
- Encrypted payload read without key fails with `KC_ENCRYPTION_REQUIRED`.
- Canonical Rust gates remain green.

## Rollout gate
- Active when M1 gates pass and schema registry promotes encryption draft row to active runtime row.

## Stop conditions
- Any change to `object_hash` derivation from plaintext.
- Any runtime path decrypts encrypted payload without key context.
- Any schema change without `SCHEMA_REGISTRY.md` update and schema validation tests.
