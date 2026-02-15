# Dependency Migration Notes

## Current status
- RustSec vulnerabilities: `0`.
- RustSec advisory warnings are policy-gated by `security/rustsec-policy.json`.
- CI gate: `.github/workflows/security-audit.yml` job `rustsec-policy`.
- Weekly sweep: same workflow job `dependency-sweep` (`cargo update` + policy check).
- Upstream release watch: `security/dependency-watch.json` + `scripts/dependency-watch.mjs`.

## Why warnings remain
- `tauri` / `tauri-utils` chain currently brings gtk3/unic advisories transitively.
- `lancedb` / `lance` 2.0.x currently pins `tantivy` 0.24.x, which pulls `lru` 0.12.x.

## Reduction strategy
1. Keep Tauri features minimal in `apps/desktop/src-tauri/Cargo.toml`.
2. Track upstream releases of:
   - `tauri`, `tauri-utils`
   - `lancedb`, `lance`, `lance-index`, `tantivy`
3. Re-run local sweep:
   - `pnpm deps:sweep`
4. Watch upstream releases:
   - strict: `pnpm deps:watch`
   - advisory: `pnpm deps:watch:advisory`
5. If advisory set changes:
   - remove stale allowlist entries
   - add new entries only with reason + short review deadline
6. Verify full workspace:
   - `pnpm lint`
   - `pnpm test`
   - `pnpm -C apps/desktop/ui build`
   - `cargo fmt --all -- --check`
   - `cargo test --workspace`
   - `cargo build --workspace`
