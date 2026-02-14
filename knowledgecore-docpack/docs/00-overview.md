# KnowledgeCore Desktop Overview

## Purpose
Project overview: goals, success metrics, constraints, determinism tiers, phase map, stop/go criteria, and explicit deferrals.

## Invariants
- Offline-first; no external APIs.
- Inspectable storage: SQLite + content-addressed object store.
- Canonical text is ground truth for chunking/indexing/citations/snippets.
- Determinism tiers enforced.
- Ask mode is retrieved-only and citation-enforced.

## Acceptance Tests
- Milestones are executable via `plan/*`.
- CLI parity is achieved before UI build.

## Goal
KnowledgeCore Desktop is a local-first RAG desktop app with a hardened deterministic Rust truth layer (IDs, canonicalization, chunking, indexes, merge, locators, export/verifier, events, Ask enforcement). Desktop UI is full-feature but thin.

## Determinism tiers
- Tier 1 strict deterministic: must be identical for given vault state.
- Tier 2 toolchain-scoped: PDF/OCR deterministic within pinned toolchain; changes define boundary.
- Tier 3 performance: measured baselines and regressions.

## Phase map
0, A–K as defined in `plan/00-milestones-and-gates.md`.

## Stop/Go
- STOP on Tier 1 invariant violation, boundary violation, or unversioned schema change.
- GO only when gates pass.

## Deferrals
- Encryption at rest
- Deterministic ZIP packaging
- Cross-device sync
