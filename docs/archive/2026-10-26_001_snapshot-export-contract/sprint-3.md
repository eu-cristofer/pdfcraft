# Sprint 3 — Internal export model and ordering

## Goal

Implement the internal export data model and deterministic ordering/ID assignment used by all later export artifacts, without yet completing the full package writer.

## Files

- the main export implementation files for snapshot/report export
- related model/utility modules for ordered items, IDs, normalized paths, and validation helpers
- focused tests for ordering and item normalization

## Reuse

- Reuse existing snapshot evidence types, report item models, and any existing selection/report ordering metadata.
- Reuse existing path-confinement and validation helpers where appropriate rather than inventing new path-normalization rules.
- Reuse the crate’s existing error types and `Result` conventions.

## Tests

- Unit tests for deterministic ordering.
- Unit tests for `item_N` assignment from a stable ordered list.
- Unit tests for normalized relative file paths and rejection of invalid/escaping paths.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p <relevant-crate> -j 2 --all-targets -- -D warnings`
- `cargo test -p <relevant-crate> -j 2`

## Definition of done

- Export items are normalized into one shared ordered list.
- IDs and future image paths derive from that ordered list only.
- Invalid pages/rectangles/paths fail through the crate’s normal error path, not panics.
- The code is structured so manifest generation and LaTeX generation can consume the same normalized item list directly.

## Assumes

Sprint 2 defined the contract the implementation must follow.
