# Identity and Hashing

## Purpose
Defines Tier 1 identity and hashing: object hashes, doc_id derivation, canonical_hash, chunking_config_hash, chunk_id derivation, and string encoding rules.

## Invariants
- Tier 1: all hashes are BLAKE3 over exact bytes.
- `object_hash = BLAKE3(file_bytes)` encoded as `blake3:<lowerhex>`.
- `doc_id` derived from original bytes only (default: equals `object_hash`).
- `canonical_hash = BLAKE3(canonical_text_bytes)`; invariant equals canonical object hash.
- `chunking_config_hash = BLAKE3(canonical_json(config))`.
- `chunk_id` uses domain-separated concatenation of stable fields.

## Acceptance Tests
- Hash vector tests for known byte inputs.
- Doc identity tests: path change does not change doc_id.
- Canonical hash invariant tests: canonical_hash equals object hash of canonical text bytes.

## Encodings
- External (JSON): `blake3:<lowerhex>`
- SQLite: store as TEXT in same format.

## Derivations (v1)
- `object_hash(bytes) -> blake3:<hex>`
- `doc_id(bytes) -> blake3:<hex>` (default: same as object_hash)
- `canonical_hash(canon_bytes) -> blake3:<hex>`
- `chunking_config_hash(config_json) -> blake3:<hex>`
- `chunk_id(doc_id, cfg_hash, ordinal, start, end) -> blake3:<hex>` using:
  - `BLAKE3("kc.chunk.v1\n" + doc_id + "\n" + cfg_hash + "\n" + ordinal + "\n" + start + ":" + end)`

## Version boundary behavior
- Any change to derivations or encodings is Tier 1 breaking and requires snapshots update.

## Error codes
- `KC_HASH_INVALID_FORMAT`
- `KC_HASH_DECODE_FAILED`
