# Chunking v1

## Purpose
Deterministic chunking algorithms, config hash, chunk IDs, and tie-break rules.

## Invariants
- Tier 1: char-index ranges into canonical text; config hash included; marker-aware.
- Any config change => new chunk IDs.

## Acceptance Tests
- Golden chunk snapshot tests pass for corpus v1.

## Char indexing (locked assumption)
- Unicode scalar indices (Rust `.chars()` indices).

## Config v1 default
```json
{"v":1,"md_html":{"max_chars":2400,"min_chars":600},"pdf":{"window_chars":1800,"overlap_chars":240,"respect_markers":true}}
```

## MD/HTML
- Use heading markers as section boundaries then split by blank lines/sentence ends.

## PDF
- Fixed window with overlap; avoid splitting inside marker lines.

## Tie-break chain (splits)
- prefer blank line boundary over sentence end; pick latest boundary <= max_chars; else hard split.

## Error codes
- `KC_CHUNKING_FAILED`
- `KC_CHUNK_CONFIG_INVALID`
