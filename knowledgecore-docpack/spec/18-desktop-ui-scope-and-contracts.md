# Desktop UI Scope and Contracts

## Purpose
Full UI feature list and hard boundary rules.

## Invariants
- UI has no business logic; branches on AppError.code only; uses RPC types only.

## Acceptance Tests
- UI smoke + unit tests pass; boundary review passes.

## Full UI scope
- Vault mgmt, ingest, search, doc view, related items, ask, exports/verifier, events, settings.

## Boundary rules
- UI must not rerank results or compute any scores.
- UI must not resolve locators; it calls RPC.
