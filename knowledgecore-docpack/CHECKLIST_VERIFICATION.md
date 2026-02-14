# CHECKLIST_VERIFICATION.md

## Purpose
Concise verification checklist for each milestone to enforce determinism, schema correctness, and boundary compliance.

## Invariants
- Tier 1 outputs must remain deterministic.
- Tier 2 outputs must be version-bounded with pinned toolchain recorded.
- UI contains no business logic; Tauri is thin RPC.
- Verification commands must be executed exactly as written.

## Acceptance Tests
- Checklist completed in each PR description.
- All required gates pass.

## Global gates (run unless milestone says otherwise)
- Rust: `cargo test -p kc_core -p kc_extract -p kc_index -p kc_ask -p kc_cli`
- Desktop (UI milestones): `pnpm lint && pnpm test && pnpm tauri build`

## Tier 1 determinism checklist
- [ ] `object_hash` is BLAKE3 of bytes; encoding `blake3:<lowerhex>`
- [ ] `doc_id` derived from bytes only (path ignored)
- [ ] Canonical JSON hashing follows `spec/00-canonical-json.md`
- [ ] `canonical_hash` is BLAKE3 of canonical text bytes
- [ ] `chunking_config_hash` = BLAKE3(canonical-json(config))
- [ ] `chunk_id` derivation matches `spec/01` and `spec/06`
- [ ] Retrieval ordering tie-break chain matches `spec/09`
- [ ] Export manifest ordering matches `spec/12`
- [ ] Verifier exit codes and report ordering match `spec/13`
- [ ] Locator strict resolution matches `spec/10`
- [ ] Lineage query node/edge ordering matches `spec/30`

## Tier 2 toolchain checklist
- [ ] PDFium identity captured and stored
- [ ] Tesseract identity + traineddata hashes captured (if OCR used)
- [ ] OCR trigger metric deterministic and versioned
- [ ] Tool/version change triggers explicit version boundary behavior

## Schema checklist
- [ ] Any schema changes updated in `SCHEMA_REGISTRY.md`
- [ ] Schema validation tests added/updated and pass
- [ ] SQLite migration version bumps correct; migration tests pass

## UI boundary checklist (UI milestones only)
- [ ] UI branches only on `AppError.code`
- [ ] UI has zero ranking/merge/chunking/locator logic
- [ ] UI lineage rendering preserves RPC ordering (no client-side reordering)
- [ ] RPC request/response types match `spec/19`
- [ ] Desktop gates pass

## PR checklist snippet (copy)
- Milestone:
- Gates executed:
- Failures encountered and fixes:
- Schemas changed:
- Determinism impacts:
- Follow-ups deferred:
