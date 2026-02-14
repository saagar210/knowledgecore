# Cross-Device Passphrase Trust v1

## Purpose
Define passphrase-only trust metadata and validation rules for multi-device sync targets.

## Invariants
- Trust model is passphrase-only in this version.
- No device certificate enrollment or external key exchange.
- Sync apply must hard-fail on trust fingerprint mismatch.
- Trust metadata serialization is deterministic.

## Non-goals
- PKI/device enrollment.
- Hardware-backed attestation.
- External identity provider integration.

## Interface contracts
- Sync head/manifest trust fields:
  - `trust.model: "passphrase_v1"`
  - `trust.fingerprint: String` (`blake3:<hex>`)
  - `trust.updated_at_ms: i64`
- Validation:
  - local derived fingerprint must equal remote trust fingerprint before pull/push commit.

## Determinism and version-boundary rules
- Fingerprint derivation input is versioned and stable.
- Any change to derivation input format requires boundary bump.
- Error behavior for mismatch is stable and non-retryable by default.

## Failure modes and AppError mapping
- `KC_SYNC_KEY_MISMATCH`: trust fingerprint mismatch.
- `KC_SYNC_AUTH_FAILED`: credentials missing/invalid for remote target.
- `KC_SYNC_NETWORK_FAILED`: transport unavailable.

## Acceptance tests
- Matching passphrase fingerprints allow push/pull.
- Mismatched fingerprints hard-fail with `KC_SYNC_KEY_MISMATCH`.
- Deterministic trust payload serialization and ordering.

## Rollout gate and stop conditions
### Rollout gate
- Trust metadata integrated in O2 and verified in CLI/RPC tests.

### Stop conditions
- Pull/push proceeds when trust fingerprint mismatches.
- Fingerprint derivation not deterministic.
