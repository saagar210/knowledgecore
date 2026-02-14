# Device Trust Manual Verify v1

## Purpose
Define manual device-key trust contracts for sync author verification using explicit fingerprint confirmation.

## Invariants
- Trust model is device-key based, local-first, and manually verified.
- Device public-key fingerprints are deterministic and human-verifiable.
- Unverified devices cannot author accepted remote sync heads.
- Trust events are append-only and deterministic in ordering.

## Non-goals
- Managed identity providers.
- PKI/certificate authority chains.
- Automatic trust enrollment.

## Interface contracts
- Core trust operations:
  - `trust_device_init`
  - `trust_device_list`
  - `trust_device_verify`
- Device key model:
  - Algorithm: `ed25519`
  - Fingerprint: `sha256(pubkey)` canonical grouped hex string
- Sync author fields on head payload:
  - `author_device_id: String`
  - `author_fingerprint: String`
  - `author_signature: String`
- CLI surface:
  - `kc_cli trust device init <vault_path> --device-label <label>`
  - `kc_cli trust device list <vault_path>`
  - `kc_cli trust device verify <vault_path> --device-id <id> --fingerprint <fp>`

## Determinism and version-boundary rules
- Fingerprint formatting is stable and versioned.
- Trust list ordering is deterministic: `created_at_ms`, then `device_id`.
- Trust event ordering is deterministic: `event_id` ascending.
- Any change to fingerprint format, signing payload, or author fields requires version boundary review.

## Failure modes and AppError mapping
- `KC_TRUST_DEVICE_UNVERIFIED`: operation requires verified device trust.
- `KC_TRUST_FINGERPRINT_MISMATCH`: provided fingerprint does not match stored key.
- `KC_SYNC_AUTH_FAILED`: signing/verification material unavailable.
- `KC_SYNC_KEY_MISMATCH`: remote author fingerprint mismatch.

## Acceptance tests
- Device init generates stable key metadata and deterministic fingerprint format.
- Verify succeeds on exact fingerprint and fails with mismatch code.
- Sync push/pull rejects unverified author and mismatched fingerprints.
- Trust schema validation tests pass.

## Rollout gate and stop conditions
### Rollout gate
- S1 gates pass canonical Rust + schema tests.

### Stop conditions
- Any sync path accepts unverified author metadata.
- Non-deterministic fingerprint or trust payload serialization.
- Missing schema registry update and tests for trust contracts.
