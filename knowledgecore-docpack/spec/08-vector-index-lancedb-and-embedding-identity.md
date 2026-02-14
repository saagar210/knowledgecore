# Vector Index (LanceDB) and Embedding Identity

## Purpose
LanceDB schema contract and embedding identity pinning.

## Invariants
- Tier 1: identity pinned; merge ordering deterministic.
- Tier 2: embeddings stable within pinned runtime on same machine.

## Acceptance Tests
- Identity recorded; rebuild/query tests pass.

## Table schema (v1) (assumption)
- chunks_vectors_v1(chunk_id, doc_id, ordinal, vector[dims], source_kind, chunking_config_hash, model_id, model_hash)

## Identity fields
- model_id, model_hash, dims, distance=cosine, provider name/version, flags_json (canonical JSON)

## Error codes
- `KC_VECTOR_INDEX_INIT_FAILED`
- `KC_VECTOR_QUERY_FAILED`
- `KC_EMBEDDING_FAILED`
