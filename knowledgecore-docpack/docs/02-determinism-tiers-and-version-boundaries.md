# Determinism Tiers and Version Boundaries

## Purpose
Precise Tier 1/2/3 definitions and acceptance tests that prove determinism and stable ordering.

## Invariants
- Tier 1 outputs must be identical for given vault state.
- Tier 2 outputs are version-bounded by toolchain identity.
- Tier 3 performance measured; does not affect correctness.

## Acceptance Tests
- Determinism and boundary tests exist and pass; golden snapshots stable under pinned toolchain.

## Tier 1 acceptance suite
- canonical JSON vectors
- hashing vectors
- chunking snapshots
- retrieval ordering snapshots (fixed now_ms)
- manifest ordering snapshots
- verifier report ordering snapshots

## Tier 2 acceptance suite
- PDFium identity capture
- Tesseract identity + traineddata hash capture
- OCR trigger determinism tests

## Version boundary rule
- Tool/version change => record new toolchain identity; treat as new boundary; snapshots updated explicitly.
