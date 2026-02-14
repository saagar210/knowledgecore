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

## Error codes
- `KC_PDFIUM_UNAVAILABLE`
- `KC_TESSERACT_UNAVAILABLE`
