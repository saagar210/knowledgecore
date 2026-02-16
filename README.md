# KnowledgeCore Development

## Canonical Commands

The canonical workspace commands are defined in:

- `package.json` scripts (`pnpm dev`, `pnpm dev:lean`, `pnpm lint`, `pnpm test`, `pnpm tauri`)
- `.github/workflows/ci.yml` (`pnpm install --frozen-lockfile`, `pnpm lint`, `pnpm test`, `pnpm -C apps/desktop/ui build`, Rust checks)

## Normal Dev

Install dependencies:

```bash
pnpm install --frozen-lockfile
```

Start desktop app (normal mode):

```bash
pnpm dev
```

## Lean Dev (Low Disk)

Start desktop app in lean mode:

```bash
pnpm dev:lean
```

What lean mode changes:

- uses a temporary `CARGO_TARGET_DIR` outside the repo for Rust build artifacts
- starts the app with the same canonical command path (`pnpm tauri dev`)
- automatically removes heavy build artifacts when the app exits

Tradeoff:

- lower persistent disk usage
- slower startup/compile on each run because build caches are ephemeral

## Cleanup Commands

Targeted cleanup (heavy build artifacts only; keeps dependencies for faster next start):

```bash
pnpm clean:heavy
```

Full local cleanup (all reproducible local caches; next run will reinstall/rebuild more):

```bash
pnpm clean:full
```

Dry-run previews:

```bash
pnpm clean:heavy:dry
pnpm clean:full:dry
```

Size audit:

```bash
pnpm size:audit
```
