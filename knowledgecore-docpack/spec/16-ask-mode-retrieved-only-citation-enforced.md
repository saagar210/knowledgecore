# Ask Mode (Retrieved-only, Citation-enforced)

## Purpose
Ask mode contract: retrieved-only generation, citation parsing, strict validation, and hard-fail behavior.

## Invariants
- Retrieved-only; no general knowledge.
- ≥1 citation per paragraph.
- Any invalid citation hard-fails entire answer.

## Acceptance Tests
- Missing citations fails; invalid citation fails; trace written and schema-valid.

## Citation format (assumption v1)
- Model output must include a machine-readable JSON block:
  - `citations: [{ paragraph_index: number, locators: [LocatorV1...] }]`

## Error codes
- `KC_ASK_MISSING_CITATIONS`
- `KC_ASK_INVALID_CITATIONS`
- `KC_ASK_PROVIDER_UNAVAILABLE`
