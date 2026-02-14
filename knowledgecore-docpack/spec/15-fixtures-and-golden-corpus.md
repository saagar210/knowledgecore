# Fixtures and Golden Corpus (v1)

## Purpose
Golden corpus v1 definition and snapshot expectations.

## Invariants
- Tier 1 snapshots exact; Tier 2 snapshots stable under pinned toolchain.
- Tests use fixed now_ms.

## Acceptance Tests
- `kc_cli fixtures generate --corpus v1` and golden tests pass.

## Fixture list (minimum)
- MD: 2 docs with nested headings
- HTML Confluence: 2 pages
- PDF: 3 docs (clean, messy, scanned/no-text)

## Expected outputs
- canonical_text, chunks, retrieval order, export manifest, verifier report

## Commands
- generate: `kc_cli fixtures generate --corpus v1`
- verify: cargo golden tests across crates
