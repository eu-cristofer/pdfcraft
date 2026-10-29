# Sprint 5 — End-to-end tests and polish

## Goal

Finish the feature to a maintainable shipped state: tighten edge-case coverage, document the user-visible behavior, and leave clean v1 boundaries for future non-rectangle or non-snapshot evidence.

## Files

- export-related engine/automation/UI tests
- final docs updates in `docs/` and any relevant crate README
- `CHANGELOG.md`
- `README.md` only if the user-facing feature should be discoverable there now

## Reuse

- Reuse the implementation from Sprints 2–4.
- Reuse existing docs style and test naming conventions.
- Reuse existing selected-comment and export notification language where possible.

## Tests

- Coverage for rotated pages and displayed-rect semantics through the full selected-comment flow.
- Coverage for deterministic package output across repeated runs.
- Coverage for mixed supported/unsupported selected comments according to the chosen v1 product rule.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-engine -j 2 --all-targets -- -D warnings`
- `cargo clippy -p pdfcraft-automation -j 2 --all-targets -- -D warnings`
- `cargo clippy -p pdfcraft-ui-egui -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-engine -j 2`
- `cargo test -p pdfcraft-automation -j 2`
- `cargo test -p pdfcraft-ui-egui -j 2 comments`

## Definition of done

- The repo documents the shipped v1 selected-rectangle export behavior clearly enough for future contributors to extend it safely.
- Tests protect the core guarantees: selected rectangle mapping, ordering, IDs, file references, `report.tex` structure, and failure behavior.
- The implementation leaves explicit seams for future evidence sources and kinds without claiming them as shipped.

## Assumes

Sprint 4 wired the selected-rectangle snapshot export flow into the desktop UI on top of the headless path.
