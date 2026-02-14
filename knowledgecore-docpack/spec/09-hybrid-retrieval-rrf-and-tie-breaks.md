# Hybrid Retrieval (RRF) and Tie-breaks

## Purpose
RRF merge formula, priors, bounded recency boost, float rounding, deterministic final ordering.

## Invariants
- Tier 1: deterministic scoring and ordering.
- Tie-break chain fixed.
- Recency uses stored effective_ts.

## Acceptance Tests
- Golden ordering snapshots pass (fixed now_ms).

## RRF
- rrf = 1/(k+r), default k=60; weights w_lex=w_vec=1.0

## Priors (caps)
- manuals 1.10, confluence_exports 1.07, notes 1.05, evidence_packs 1.08, inbox 0.98, other 1.00
- cap range [0.90,1.15] (assumption)

## Recency (configurable)
- enabled flag present; default disabled.
- window_days=30, max_boost=0.03

## Ordering
- round(final_score,12) desc, doc_id asc, ordinal asc, chunk_id asc

## Error codes
- `KC_RETRIEVAL_MERGE_FAILED`
- `KC_RETRIEVAL_PRIOR_OUT_OF_RANGE`
