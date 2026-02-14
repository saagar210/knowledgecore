# Module Boundaries

## Purpose
Crate responsibilities, allowed/forbidden dependencies, and ownership of truth-layer rules.

## Invariants
- `kc_core` is the sole owner of Tier 1 deterministic rules.
- `kc_extract` produces canonical text artifacts (Tier 2).
- `kc_index` provides FTS and vector candidate services.
- `kc_ask` performs Ask mode using core APIs.
- `kc_cli` is automation; no truth rules.
- UI/Tauri contain no business logic.

## Acceptance Tests
- Code review and tests confirm no cycles and no UI business logic.

## Forbidden dependencies
- `kc_core` must not depend on `kc_extract`, `kc_index`, `kc_ask`, or UI.
- Tauri must not run direct SQLite queries bypassing core.
- UI must not implement ranking/merge/chunking/locator/export/verifier logic.

## Ownership map
- IDs/hashing/canonical JSON: kc_core
- canonical persistence + registry: kc_core
- extraction + OCR: kc_extract
- index implementations: kc_index
- merge ordering: kc_core
- locators: kc_core
- export manifest ordering: kc_core
- verifier: kc_cli (must conform)

## Error boundaries
- All layers communicate via AppError; UI branches only on AppError.code.
