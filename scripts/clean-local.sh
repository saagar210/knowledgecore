#!/usr/bin/env bash
set -euo pipefail

DRY_RUN=0
AGGRESSIVE=0
PRUNE_NODE_MODULES=0
TARGET_THRESHOLD_GB=0

usage() {
  cat <<'EOF'
Usage: scripts/clean-local.sh [options]

Safe defaults:
  - cleans Cargo dev artifacts
  - prunes release intermediates (deps/build/.fingerprint)
  - prunes tauri-dist output
  - removes .DS_Store files

Options:
  --dry-run                 Show what would be cleaned
  --aggressive              Also clean Cargo release artifacts
  --prune-node-modules      Also clear node_modules directories
  --target-threshold-gb N   Skip Cargo target cleanup unless target >= N GB
  -h, --help                Show this help
EOF
}

while (($# > 0)); do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      ;;
    --aggressive)
      AGGRESSIVE=1
      ;;
    --prune-node-modules)
      PRUNE_NODE_MODULES=1
      ;;
    --target-threshold-gb)
      shift
      TARGET_THRESHOLD_GB="${1:-}"
      if [[ -z "$TARGET_THRESHOLD_GB" || ! "$TARGET_THRESHOLD_GB" =~ ^[0-9]+$ ]]; then
        echo "error: --target-threshold-gb requires a non-negative integer" >&2
        exit 1
      fi
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option '$1'" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

human_kb() {
  awk -v kb="$1" '
    BEGIN {
      split("KB MB GB TB PB", u, " ");
      v = kb + 0;
      i = 1;
      while (v >= 1024 && i < 5) {
        v /= 1024;
        i++;
      }
      printf "%.2f %s", v, u[i];
    }'
}

dir_kb() {
  local path="$1"
  if [[ -e "$path" ]]; then
    du -sk "$path" 2>/dev/null | awk '{print $1}'
  else
    echo 0
  fi
}

count_entries() {
  local path="$1"
  if [[ -d "$path" ]]; then
    find "$path" -mindepth 1 2>/dev/null | wc -l | tr -d ' '
  else
    echo 0
  fi
}

prune_dir_contents() {
  local path="$1"
  if [[ ! -d "$path" ]]; then
    return
  fi

  if ((DRY_RUN)); then
    echo "[dry-run] would prune $path ($(count_entries "$path") entries)"
  else
    find "$path" -mindepth 1 -delete
    echo "pruned $path"
  fi
}

run_cargo_clean_dev() {
  if ((DRY_RUN)); then
    cargo clean --dry-run --profile dev
    return
  fi

  if cargo clean --profile dev; then
    return
  fi

  # Cargo can hit transient macOS directory races in large incremental dirs.
  echo "retrying cargo clean --profile dev..."
  cargo clean --profile dev
}

run_cargo_clean_release() {
  if ((DRY_RUN)); then
    cargo clean --dry-run --release
  else
    cargo clean --release
  fi
}

echo "Repository: $ROOT"
before_repo_kb="$(du -sk . | awk '{print $1}')"
before_target_kb="$(dir_kb target)"
echo "Before: repo=$(human_kb "$before_repo_kb"), target=$(human_kb "$before_target_kb")"

should_clean_target=1
if ((TARGET_THRESHOLD_GB > 0)); then
  threshold_kb=$((TARGET_THRESHOLD_GB * 1024 * 1024))
  if ((before_target_kb < threshold_kb)); then
    should_clean_target=0
  fi
fi

if ((should_clean_target)); then
  run_cargo_clean_dev
else
  echo "target below threshold (${TARGET_THRESHOLD_GB}GB), skipping Cargo target clean"
fi

# Keep release bundles/executables, but drop heavyweight intermediates.
prune_dir_contents "target/release/deps"
prune_dir_contents "target/release/build"
prune_dir_contents "target/release/.fingerprint"
prune_dir_contents "apps/desktop/ui/tauri-dist"

if ((AGGRESSIVE)); then
  run_cargo_clean_release
fi

if ((PRUNE_NODE_MODULES)); then
  prune_dir_contents "node_modules"
  prune_dir_contents "apps/desktop/ui/node_modules"
fi

if ((DRY_RUN)); then
  ds_count="$(find . -type f -name '.DS_Store' 2>/dev/null | wc -l | tr -d ' ')"
  echo "[dry-run] would remove $ds_count .DS_Store files"
else
  find . -type f -name '.DS_Store' -delete
  echo "removed .DS_Store files"
fi

after_repo_kb="$(du -sk . | awk '{print $1}')"
after_target_kb="$(dir_kb target)"
reclaimed_kb=$((before_repo_kb - after_repo_kb))

echo "After:  repo=$(human_kb "$after_repo_kb"), target=$(human_kb "$after_target_kb")"
echo "Reclaimed: $(human_kb "$reclaimed_kb")"

if ((DRY_RUN)); then
  echo "Dry-run complete. Re-run without --dry-run to apply changes."
fi
