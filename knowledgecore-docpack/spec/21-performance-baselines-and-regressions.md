# Performance Baselines and Regressions (Tier 3)

## Purpose
Bench harness and regression thresholds.

## Invariants
- Tier 3 only; does not affect Tier 1 behavior; offline benches.

## Acceptance Tests
- Harness exists; smoke thresholds defined.

## Bench harness
- criterion benches in kc_core/kc_index
- `kc_cli bench run --corpus v1`

## Thresholds (initial)
- Fail if >3x baseline once baselines recorded.
