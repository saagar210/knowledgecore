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

## Baseline persistence
- Baseline file path: `.bench/baseline-v1.json`
- Stored fields:
  - `corpus`
  - `elapsed_ms`
  - `checksum`
- First run creates baseline; subsequent runs compare against `elapsed_ms * 3`.

## Corpus v1 workload assumptions
- Uses deterministic core workload primitives (canonical hashing + chunking) across fixed synthetic corpus docs.
- Produces a deterministic checksum; checksum drift indicates workload implementation drift.
- Intended as smoke-level regression detection, not hardware-normalized benchmarking.
