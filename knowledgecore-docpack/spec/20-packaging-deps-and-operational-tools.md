# Packaging, Dependencies, and Operational Tools

## Purpose
Dev vs release toolchain strategy and operational CLI tools.

## Invariants
- Offline-first; toolchain identities recorded; ops tools emit events; no silent mutation.

## Acceptance Tests
- Dependency checks return actionable errors; ops tools tested.

## Dev vs Release (assumption)
- Dev: system-installed PDFium/Tesseract allowed with configured paths.
- Release: bundle tools preferred; otherwise guided installer flow.

## Ops tools (CLI)
- vault verify, index rebuild, gc run, export subset

## Error codes
- `KC_PDFIUM_UNAVAILABLE`
- `KC_TESSERACT_UNAVAILABLE`
