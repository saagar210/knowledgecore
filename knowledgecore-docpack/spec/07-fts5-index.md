# FTS5 Index Contract

## Purpose
FTS5 schema, tokenizer, deterministic rebuild rules, and error codes.

## Invariants
- Tier 1: indexed content derived from canonical substring; rebuild deterministic.

## Acceptance Tests
- Rebuild and query tests pass; stable candidate set for fixed corpus.

## DDL
```sql
CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts
USING fts5(chunk_id UNINDEXED, doc_id UNINDEXED, content, tokenize='unicode61');
```

## Rebuild order (Tier 1)
- Insert in order: doc_id asc, ordinal asc, chunk_id asc.

## Error codes
- `KC_FTS_INIT_FAILED`
- `KC_FTS_REBUILD_FAILED`
- `KC_FTS_QUERY_FAILED`
