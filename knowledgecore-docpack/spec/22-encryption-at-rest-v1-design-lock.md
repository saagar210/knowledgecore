# Encryption at Rest v1 (Design Lock)

## Purpose
Define Phase M implementation boundaries for optional encryption at rest without changing runtime behavior in Phase L.

## Invariants
- Encryption-at-rest remains deferred during Phase L.
- No default runtime behavior changes are allowed in `kc_core`, `kc_cli`, Tauri, or UI.
- Any future encryption metadata that affects Tier 1 ordering must be canonicalized and versioned explicitly.
- UI continues branching only on `AppError.code`.

## Non-goals
- No active object encryption writes.
- No active SQLite encryption migration.
- No passphrase UX delivery in desktop UI.
- No key escrow, remote KMS, or cross-device key exchange.

## Interface Contracts (Draft)
### `EncryptionMetadataDraftV1`
- `schema_version: i64` (const `1`)
- `status: String` (const `"draft"`)
- `activation_phase: String` (const `"M"`)
- `cipher_suite: String` (planned default `"xchacha20poly1305"`)
- `kdf: KdfParamsDraftV1`
- `key_reference: String` (non-secret reference only)

### `KdfParamsDraftV1`
- `algorithm: String` (planned default `"argon2id"`)
- `memory_kib: i64`
- `iterations: i64`
- `parallelism: i64`
- `salt_id: String` (non-secret identifier)

### `EncryptionMigrationPlanDraftV1`
- `source_mode: String` (`"plaintext"`)
- `target_mode: String` (`"encrypted"`)
- `verification_required: bool` (must be `true`)
- `rollback_supported: bool` (must be `true`)

### Draft RPC and CLI shell contracts
- Preview RPC method (feature-gated): `preview_encryption_status`
- Preview CLI shell (feature-gated): `kc_cli preview capability --name encryption`

## Determinism and Version-Boundary Rules
- Metadata serialization must use canonical JSON before hashing or signing.
- Planned encrypted object manifests must preserve deterministic object ordering from `spec/12`.
- Any cipher, KDF, or parameter default change establishes a version boundary and requires explicit snapshot updates.
- Toolchain identity for encryption library/version is Tier 2-adjacent and must be recorded once implementation begins.

## Failure Modes and AppError Code Map (Draft)
- `KC_DRAFT_ENCRYPTION_NOT_IMPLEMENTED`: preview shell reached; runtime behavior intentionally absent.
- `KC_DRAFT_ENCRYPTION_KEY_DERIVATION_UNIMPLEMENTED`: KDF operation requested outside activated phase.
- `KC_DRAFT_ENCRYPTION_MIGRATION_UNIMPLEMENTED`: migration command path invoked before Phase M promotion.
- `KC_DRAFT_PREVIEW_UNKNOWN_CAPABILITY`: preview shell requested unsupported capability name.

## Acceptance Tests (Phase L)
- Schema validation test for `EncryptionMetadataDraftV1` exists and passes.
- Feature-disabled builds expose no encryption runtime behavior.
- Feature-enabled preview scaffold emits deterministic placeholder payloads/errors.
- Canonical gates remain green.

## Rollout Gate and Stop Conditions
### Rollout gate to start Phase M
- Security design review approved.
- Key loss/recovery runbook approved.
- Schema registry draft entry promoted to active implementation entry.

### Stop conditions
- Any runtime encryption behavior is introduced in Phase L.
- Any schema draft lacks validation tests.
- Any boundary violation appears in UI or Tauri layers.
