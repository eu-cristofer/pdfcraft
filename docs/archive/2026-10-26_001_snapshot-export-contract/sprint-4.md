# Sprint 4 — Manifest and LaTeX generation core

## Goal

Generate the canonical `manifest.json` and deterministic snapshot `.tex` output from the normalized item list, with exact ordering and exact template structure.

## Files

- export writer/generator modules for manifest serialization and LaTeX rendering
- tests/golden fixtures for manifest and LaTeX output
- any tiny doc updates that explain the produced files

## Reuse

- Reuse the normalized item model from Sprint 3.
- Reuse the project’s existing LaTeX block structure exactly where required.
- Reuse existing serialization helpers and test fixture conventions.

## Tests

- Golden test for manifest output.
- Golden test for LaTeX output.
- Consistency test ensuring every manifest item yields exactly one LaTeX block with matching label/path.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p <relevant-crate> -j 2 --all-targets -- -D warnings`
- `cargo test -p <relevant-crate> -j 2`

## Definition of done

- `manifest.json` is produced from the normalized item list and follows the v1 contract.
- `.tex` is produced from the same ordered item list and preserves the exact required snapshot block syntax.
- Manifest order, item IDs, LaTeX labels, and image references stay aligned in tests.

## Assumes

Sprint 3 introduced the normalized item model and deterministic ordering/ID helpers.
