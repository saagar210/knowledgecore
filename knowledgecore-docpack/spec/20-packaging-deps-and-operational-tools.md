# Packaging, Dependencies, and Operational Tools

## Purpose
Dev vs release toolchain strategy and operational CLI tools.

## Invariants
- Offline-first; toolchain identities recorded; ops tools emit events; no silent mutation.

## Acceptance Tests
- Dependency checks return actionable errors; ops tools tested.
- `kc_cli deps check` reports discovered tool identities.
- `kc_cli index rebuild <vault_path>` rebuilds lexical and vector indexes deterministically.
- `kc_cli bench run --corpus v1` creates/uses baseline and fails when elapsed time exceeds 3x baseline.

## Dev vs Release (assumption)
- Dev: system-installed PDFium/Tesseract allowed with configured paths.
- Release: bundle tools preferred; otherwise guided installer flow.

## Ops tools (CLI)
- vault verify, index rebuild, gc run, export subset
- bench run with baseline threshold checking
- dependency check with version output

## Operational command contracts (v1)
- `kc_cli deps check`
  - prints tool identities (`pdftotext`, `pdftoppm`, `tesseract`) when all required tools exist
  - hard-fails with deterministic AppError codes:
    - `KC_PDFIUM_UNAVAILABLE` if PDF text/image tooling is missing
    - `KC_TESSERACT_UNAVAILABLE` if OCR tooling is missing
- `kc_cli index rebuild <vault_path>`
  - rebuilds FTS and vector artifacts deterministically from canonical/chunk rows
  - persists vectors under `index/vectors/lancedb-v1`
- `kc_cli gc run <vault_path>`
  - removes only orphan object files not referenced by the SQLite `objects` table
  - does not mutate canonical rows or chunk/index metadata
- `kc_cli vault verify <vault_path>`
  - runs SQLite integrity checks and validates required directory topology
  - prints deterministic JSON summary on success

## Desktop packaging note
- Desktop build gate uses real Tauri CLI invocation (`pnpm tauri build`) with:
  - config: `apps/desktop/src-tauri/tauri.conf.json`
  - isolated frontend dist: `apps/desktop/ui/tauri-dist`
  - generated app artifacts under `target/release/`

## Error codes
- `KC_PDFIUM_UNAVAILABLE`
- `KC_TESSERACT_UNAVAILABLE`
