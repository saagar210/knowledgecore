# Post-D–K Operations and Follow-up Policy

## Purpose
Define stable post-delivery operations policy for KnowledgeCore Desktop after Phases D–K, including benchmark baseline governance, hygiene policy, and branch lifecycle controls.

## Scope
- Applies to local development and pre-release validation runs.
- Applies to benchmark smoke policy (`kc_cli bench run --corpus v1`).
- Applies to local git hygiene and branch lifecycle after milestone merges.

## Bench Baseline Governance

### Baseline file
- Path: `.bench/baseline-v1.json`
- Schema:
  - `corpus` (string)
  - `elapsed_ms` (integer)
  - `checksum` (u64 integer)

### First run behavior
- If no baseline exists, create baseline from current deterministic workload.

### Steady-state behavior
- If baseline checksum matches runtime checksum:
  - Compare `elapsed_ms` against `baseline_ms * 3`.
  - Fail if elapsed exceeds threshold.

### Workload checksum drift behavior
- If baseline checksum differs from runtime checksum:
  - Treat as workload-version change, not immediate regression.
  - Refresh baseline in place with current elapsed/checksum.
  - Require one additional confirmation run to exercise threshold path.

### PR/Release expectations
- Any intentional workload-shape changes in bench logic must mention:
  - why checksum changed,
  - baseline refresh rationale,
  - result of second confirmation run.

## Local Git Hygiene Policy

### Safety-first sequence (mandatory before aggressive cleanup)
1. Confirm clean `master`.
2. Create safety tag `safety/pre-hygiene-<timestamp>`.
3. Create bundle backup at `.git-backups/pre-hygiene-<timestamp>.bundle`.
4. Capture branch inventory + merged/non-merged state into `.git-backups/pre-hygiene-<timestamp>.log`.

### Aggressive hygiene sequence
1. Delete all merged `codex/*` branches.
2. Run `git reflog expire --expire=now --expire-unreachable=now --all`.
3. Run `git gc --prune=now --aggressive`.
4. Run `git fsck --full`.

### Completion criteria
- `master` is the only active local branch by default.
- `git branch --list 'codex/*'` returns no branches.
- `git branch --no-merged master` returns no branches.
- `git fsck --full` returns no integrity errors.

## Branch Lifecycle Policy

### Milestone branches
- Prefix: `codex/`.
- Lifecycle: create for milestone, merge fast-forward to `master`, delete immediately after merge.

### If automated deletion is blocked
- Perform branch deletion directly in a local shell with equivalent commands.
- Record fallback usage in closure/readiness notes.

### Long-lived branches
- Avoid long-lived milestone branches once merged.
- Keep only `master` unless there is an active in-progress milestone.

## Deferred Items Carry-forward Policy
- The following remain deferred until explicitly promoted to active scope:
  - encryption at rest,
  - deterministic ZIP packaging,
  - cross-device sync,
  - advanced lineage UI.
- Promotions must include:
  - explicit phase assignment,
  - schema/API impact statement,
  - verification gates and acceptance tests.
