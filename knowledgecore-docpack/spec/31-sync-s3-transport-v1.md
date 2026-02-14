# Sync S3 Transport v1

## Purpose
Define sync transport v2 contracts that preserve local filesystem sync and add optional `s3://` target support.

## Invariants
- Local sync (`file://` and plain paths) remains supported.
- S3 sync is optional and explicitly configured.
- No auto-merge on divergence; conflict artifact emission is mandatory.
- Head, snapshot, and conflict payload serialization is deterministic.

## Non-goals
- Background sync service.
- Conflict auto-resolution.
- Device key enrollment.

## Interface contracts
- Target URI contract:
  - `file:///abs/path`
  - plain local path
  - `s3://<bucket>/<prefix>`
- Core operations:
  - `sync_status(vault_path, target_uri)`
  - `sync_push(vault_path, target_uri, now_ms)`
  - `sync_pull(vault_path, target_uri, now_ms)`
- S3 object layout:
  - `<prefix>/head.json`
  - `<prefix>/snapshots/<snapshot_id>.zip`
  - `<prefix>/conflicts/conflict_<now_ms>_<id>.json`
  - `<prefix>/locks/write.lock`

## Determinism and version-boundary rules
- `snapshot_id = blake3("kc.sync.snapshot.v2\n<manifest_hash>\n<now_ms>")`
- Conflict artifact bytes are canonical JSON.
- Conflict filename uses deterministic prefix from artifact hash.
- Any change to URI interpretation, lock semantics, or snapshot id derivation requires a version boundary update.

## Failure modes and AppError mapping
- `KC_SYNC_TARGET_UNSUPPORTED`: unsupported target URI scheme.
- `KC_SYNC_AUTH_FAILED`: remote authentication failed.
- `KC_SYNC_NETWORK_FAILED`: transport I/O/network error.
- `KC_SYNC_LOCKED`: remote lock exists and is active.
- `KC_SYNC_KEY_MISMATCH`: passphrase trust fingerprint mismatch.
- Existing retained:
  - `KC_SYNC_TARGET_INVALID`
  - `KC_SYNC_STATE_FAILED`
  - `KC_SYNC_CONFLICT`
  - `KC_SYNC_APPLY_FAILED`

## Acceptance tests
- URI parser accepts valid file/s3 targets and rejects unknown schemes.
- Local filesystem sync remains behavior-compatible.
- S3 push/pull/status deterministic behavior validated with adapter tests.
- Lock and divergence produce deterministic failure artifacts/codes.

## Rollout gate and stop conditions
### Rollout gate
- O1/O2/O3 milestones pass canonical Rust + desktop + RPC/schema gates.

### Stop conditions
- Any transport path performs silent overwrite during divergence.
- Any non-deterministic serialization of head/conflict payloads.
- Any schema-affecting change without schema registry + tests.
