# Deterministic ZIP Packaging v1 (Design Lock)

## Purpose
Define Phase N1 contracts for deterministic ZIP packaging as a transport format layered over deterministic folder export.

## Invariants
- Phase L must not ship ZIP packaging runtime behavior.
- Folder export determinism rules from `spec/12` remain source of truth.
- ZIP ordering and metadata rules must be explicit and testable before activation.

## Non-goals
- No ZIP output in default CLI commands.
- No multi-format packaging matrix.
- No compression tuning optimization work in Phase L.

## Interface Contracts (Draft)
### `ZipPackagingMetadataDraftV1`
- `schema_version: i64` (const `1`)
- `status: String` (const `"draft"`)
- `activation_phase: String` (const `"N1"`)
- `format: String` (const `"zip"`)
- `entry_order: String` (const `"lexicographic_path"`)
- `timestamp_policy: String` (const `"fixed_epoch_ms"`)
- `permission_policy: String` (const `"normalized_posix_mode"`)

### `ZipEntryDraftV1`
- `relative_path: String`
- `sha256_or_blake3: String`
- `uncompressed_bytes: i64`
- `normalized_mode: String`

### Draft RPC and CLI shell contracts
- Preview RPC method (feature-gated): `preview_zip_packaging_status`
- Preview CLI shell (feature-gated): `kc_cli preview capability --name zip_packaging`

## Determinism and Version-Boundary Rules
- Entry ordering must be deterministic and independent of filesystem traversal order.
- Entry timestamps must be normalized to a fixed value.
- Permission bits must be normalized per policy.
- Any change to ordering, timestamp, or mode policy requires a schema/version boundary update.

## Failure Modes and AppError Code Map (Draft)
- `KC_DRAFT_ZIP_PACKAGING_NOT_IMPLEMENTED`: preview shell reached; ZIP behavior intentionally absent.
- `KC_DRAFT_ZIP_POLICY_UNIMPLEMENTED`: deterministic ZIP policy requested before activation phase.
- `KC_DRAFT_PREVIEW_UNKNOWN_CAPABILITY`: preview shell requested unsupported capability name.

## Acceptance Tests (Phase L)
- Schema validation test for `ZipPackagingMetadataDraftV1` exists and passes.
- Feature-enabled preview shell returns deterministic ZIP draft metadata or draft placeholder errors.
- Default builds expose no ZIP packaging commands or RPC handlers.
- Existing export/verifier Tier 1 tests remain green.

## Rollout Gate and Stop Conditions
### Rollout gate to start Phase N1
- Deterministic ZIP fixture set approved.
- Verifier draft checks are defined and test cases accepted.
- Registry draft entry promoted to active implementation entry.

### Stop conditions
- Any active ZIP export path appears during Phase L.
- Draft metadata fields are not deterministic or not schema validated.
