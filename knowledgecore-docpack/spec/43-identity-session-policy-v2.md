# Identity Session Policy v2

## Purpose
Define deterministic policy constraints for managed identity sessions, including claim requirements, clock-skew handling, and revocation precedence.

## Invariants
- Identity sessions are accepted only when:
  - provider is enabled,
  - session is not revoked,
  - required claims policy is satisfied,
  - temporal bounds are valid within configured skew.
- Validation order is fixed and deterministic.
- Policy evaluation output is stable for equal inputs.

## Non-goals
- Expanding token cryptographic verification semantics.
- Introducing provider-specific custom policy engines.
- Changing existing sync signature chain hashing rules.

## Interface contracts
- Session policy fields:
  - `max_clock_skew_ms` (integer >= 0)
  - `require_claims_json` (canonical JSON object)
- Required claim keys supported in v2:
  - `iss`, `aud`, `sub`
- Evaluation function:
  - `evaluate_identity_session_policy(session_claim_subset_json, policy, now_ms)`

## Determinism and version-boundary rules
- Claim evaluation order is lexicographic by claim key.
- Claim objects are canonical-json normalized before compare.
- Temporal validation uses fixed formula:
  - `issued_at_ms - max_clock_skew_ms <= now_ms <= expires_at_ms + max_clock_skew_ms`
- Any change to claim key set, compare semantics, or temporal formula requires version bump.

## Failure modes and AppError code map
- `KC_TRUST_PROVIDER_POLICY_INVALID`
- `KC_TRUST_IDENTITY_INVALID`
- `KC_TRUST_SESSION_REVOKED`

## Acceptance tests
- Required-claims mismatch fails deterministically.
- Temporal skew acceptance/rejection boundaries are deterministic for fixed `now_ms`.
- Canonicalization of `require_claims_json` is stable across repeated writes.
- Revoked sessions fail before claim or time evaluation.

## Rollout gate and stop conditions
### Rollout gate
- `cargo test -p kc_core -- trust_identity`
- `cargo test -p kc_core -- schema_`
- canonical Rust gate from `knowledgecore-docpack/AGENTS.md`

### Stop conditions
- Validation order drifts (revocation no longer first).
- Time-bound validation is not deterministic for fixed inputs.
- Missing schema registry updates and tests for session policy fields.
