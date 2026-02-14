# Snippet Rendering (Display-only)

## Purpose
Display-only snippet transformations for UI/CLI.

## Invariants
- Tier 1: does not affect locators or canonical text; deterministic transforms only.

## Acceptance Tests
- Render tests pass; marker stripping does not affect locator resolution.

## Rules (v1)
- Strip marker lines `[[PAGE:...]]` and `[[Hn:...]]` for display.
- Collapse excessive blank lines.
- Trim whitespace.

## Error codes
- `KC_SNIPPET_RENDER_FAILED`
