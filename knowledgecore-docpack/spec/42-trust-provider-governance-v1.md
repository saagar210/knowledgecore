# Trust Provider Governance v1

## Purpose
Define deterministic governance for managed identity providers, including provider lifecycle, claim policy enforcement, and session revocation.

## Invariants
- Trust provider evaluation is deterministic for fixed inputs.
- Disabled providers cannot issue accepted sessions.
- Revoked sessions are excluded before claim-acceptance checks.
- UI and Tauri do not evaluate governance policy; core does.

## Non-goals
- Replacing OIDC claim normalization from existing trust identity contracts.
- Altering device certificate chain semantics.
- Introducing non-deterministic trust fallback behavior.

## Interface contracts
- Provider APIs:
  - `trust_provider_add(conn, provider_id, issuer, audience, jwks_url, now_ms)`
  - `trust_provider_disable(conn, provider_id, now_ms)`
  - `trust_provider_list(conn)`
- Policy APIs:
  - `trust_provider_policy_set(conn, provider_id, max_clock_skew_ms, require_claims_json, now_ms)`
  - `trust_provider_policy_get(conn, provider_id)`
- Session governance APIs:
  - `trust_session_revoke(conn, session_id, revoked_by, now_ms)`
  - `trust_session_is_revoked(conn, session_id)`

## Determinism and version-boundary rules
- Provider list ordering is lexicographic by `provider_id`.
- Required claims are canonical JSON and sorted by key/value pairs before persistence.
- Session selection order remains deterministic:
  - `created_at_ms DESC`, `session_id DESC`
  - revoked sessions are filtered before selection.
- Any change to claim filtering precedence or ordering rules requires version-boundary review.

## Failure modes and AppError code map
- `KC_TRUST_PROVIDER_POLICY_INVALID`
- `KC_TRUST_PROVIDER_DISABLED`
- `KC_TRUST_SESSION_REVOKED`
- Existing identity/certificate codes remain active:
  - `KC_TRUST_OIDC_PROVIDER_UNAVAILABLE`
  - `KC_TRUST_IDENTITY_INVALID`
  - `KC_TRUST_CERT_CHAIN_INVALID`

## Acceptance tests
- Provider add/disable/list round-trip is deterministic.
- Policy set/get persists canonical claims JSON and deterministic ordering.
- Revoked sessions are excluded from active-session selection.
- Disabled provider blocks identity completion with stable AppError code.

## Rollout gate and stop conditions
### Rollout gate
- `cargo test -p kc_core -- trust_identity`
- `cargo test -p kc_core -- schema_`
- canonical Rust gate from `knowledgecore-docpack/AGENTS.md`

### Stop conditions
- Any revoked session is still accepted for author identity.
- Provider policy serialization is non-deterministic.
- Schema and registry updates are missing for trust governance contracts.
