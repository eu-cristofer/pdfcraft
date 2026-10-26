# Sprint 6 — Tests, docs, and extensibility guardrails

## Goal

Finish the feature to a maintainable state: add the focused verification matrix, tighten docs, and leave explicit guardrails for future evidence kinds and output targets without prematurely generalizing the implementation.

## Files

- export-related tests and fixtures
- final docs updates in `docs/` and any relevant crate README
- changelog/README pointers if the new feature should now be visible there

## Reuse

- Reuse the feature implementation from Sprints 3–5.
- Reuse existing documentation style and test naming conventions.
- Reuse any extension-point patterns already used in this codebase rather than introducing a plugin framework.

## Tests

- Coverage for problematic text characters / LaTeX-sensitive content according to the chosen trust model.
- Coverage for deterministic outputs across repeated runs.
- Coverage for rejection of unsupported evidence kinds in v1.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p <relevant-crate> -j 2 --all-targets -- -D warnings`
- `cargo test -p <relevant-crate> -j 2`

## Definition of done

- The repo documents the v1 export behavior clearly enough for future contributors to extend it safely.
- Tests protect the core stability guarantees: ordering, IDs, file references, LaTeX structure, and validation.
- The implementation leaves clean seams for future evidence kinds and output targets without claiming them as shipped.

## Assumes

Sprint 5 completed the end-to-end package writer and its core validation.
