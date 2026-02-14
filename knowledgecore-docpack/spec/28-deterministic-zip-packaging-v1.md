# Deterministic ZIP Packaging (v1)

## Purpose
Define deterministic ZIP export behavior layered over deterministic folder export bundles.

## Invariants
- ZIP payload is generated from an already deterministic folder bundle.
- ZIP entry order is strict lexicographic order on relative path.
- ZIP compression is `stored` (no compression variance).
- ZIP entry timestamps are fixed to `1980-01-01T00:00:00Z`.
- ZIP file permissions are normalized to `0644` for file entries.
- Verifier validates ZIP metadata policy before bundle-content validation.

## Non-goals
- Streaming ZIP generation.
- Encrypted ZIP container formats.
- Cross-device transport protocols.

## Interface contract
- CLI export supports `--zip` to emit deterministic ZIP files.
- `manifest.json` includes `packaging` block:
  - `format`: `folder` or `zip`
  - `zip_policy`: `compression`, `mtime`, `file_mode`
- Verifier accepts both folder and `.zip` bundle inputs.

## Failure modes and AppError mapping
- `KC_EXPORT_FAILED` when ZIP write/finalize fails.
- `KC_VERIFY_FAILED` when ZIP extraction/validation internal operations fail.
- Verifier report codes:
  - `MANIFEST_SCHEMA_INVALID` for invalid ZIP metadata policy.
  - existing bundle data mismatch/missing codes remain unchanged.

## Acceptance tests
- Repeated ZIP exports from identical vault state are byte-identical.
- Verifier succeeds for deterministic ZIP bundles.
- Verifier fails deterministic checks for non-normalized ZIP metadata.

## Rollout gate
- N1 Rust gates pass including export/verifier tests.

## Stop conditions
- Any ZIP metadata field is host/time-dependent.
- ZIP entry ordering is not deterministic.
- Verifier allows ZIP path traversal components.
