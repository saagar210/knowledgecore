# Managed Identity OIDC + Device Certificate v1

## Purpose
Define the managed identity trust model that combines OIDC operator identity with per-device certificate enrollment for sync authorship.

## Invariants
- Local-first architecture remains intact; remote identity providers are optional adapters.
- Device authorship is accepted only when identity and certificate chain verification succeed.
- Trust verification decisions are deterministic for fixed inputs.
- Existing manual trust model remains readable for compatibility.

## Non-goals
- Replacing all local trust flows with cloud-only identity.
- Managed key escrow in this spec.
- Non-deterministic trust heuristics.

## Interface contracts
- Core trust identity APIs:
  - `trust_identity_start(vault_path, provider_id, now_ms)`
  - `trust_identity_complete(vault_path, provider_id, auth_code, now_ms)`
  - `trust_device_enroll(vault_path, device_label, now_ms)`
  - `trust_device_verify_chain(vault_path, device_id, now_ms)`
  - `trust_device_list(vault_path)`
- Sync head extensions (v3):
  - `author_device_id`
  - `author_fingerprint`
  - `author_signature`
  - `author_cert_id`
  - `author_chain_hash`
- Identity provider/session records:
  - `provider_id`
  - `issuer`
  - `subject`
  - `audience`
  - `issued_at_ms`
  - `expires_at_ms`

## Determinism and version-boundary rules
- OIDC claims used for signing input are normalized to a fixed canonical subset.
- Signature payload canonicalization uses canonical JSON rules from `spec/00-canonical-json.md`.
- Certificate chain hash derivation is deterministic.
- Any change to normalized claim subset, payload format, or chain hash derivation requires version-boundary review.

## Failure modes and AppError mapping
- `KC_TRUST_OIDC_PROVIDER_UNAVAILABLE`
- `KC_TRUST_IDENTITY_INVALID`
- `KC_TRUST_CERT_CHAIN_INVALID`
- `KC_TRUST_SIGNATURE_INVALID`
- `KC_TRUST_DEVICE_NOT_ENROLLED`

## Acceptance tests
- OIDC identity start/complete round-trip validates deterministic claim normalization.
- Enrolled device with valid chain signs and verifies successfully.
- Invalid chain or signature fails with stable AppError code.
- Sync head v3 serialization is deterministic.

## Rollout gate and stop conditions
### Rollout gate
- `cargo test -p kc_core -p kc_cli`
- canonical Rust gate from `knowledgecore-docpack/AGENTS.md`

### Stop conditions
- Any path accepts unverified identity or invalid certificate chain.
- Any non-deterministic serialization of trust identity artifacts.
- Missing schema registry update and schema validation tests.
