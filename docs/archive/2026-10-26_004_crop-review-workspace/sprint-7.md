# Sprint 7 — Export integration, OCR, and final hardening

## Goal

Wire export from the stored crop dataset, integrate OCR-backed export behavior through the shared lower-layer package writer, and finish the docs/tests so the shipped workspace is stable and clearly bounded.

## Files

- export-related engine integration points
- automation and UI call sites that should export from stored crop items
- crop-review and export tests across engine, automation, and UI
- final docs updates in `docs/` and relevant crate README text
- `CHANGELOG.md`
- `README.md` only if the new crop-review workspace should be discoverable there now

## Reuse

- Reuse the snapshot package writer, metadata mapping, and OCR integration already built below the UI layer.
- Reuse the persistent crop dataset from Sprints 2 and 6.
- Reuse the repo's existing OCR availability rule instead of inventing a second fallback path.

## Tests

- Coverage proving export runs from stored crop items rather than a transient drag.
- Coverage for deterministic export order across repeated runs.
- Coverage for rotated pages and stored geometry semantics through the full export path.
- Coverage for OCR unavailable-model failure behavior through both automation and desktop entrypoints.
- Non-regression coverage that the original snapshot tool still remains separate.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-engine -j 2 --all-targets -- -D warnings`
- `cargo clippy -p pdfcraft-automation -j 2 --all-targets -- -D warnings`
- `cargo clippy -p pdfcraft-ui-egui -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-engine -j 2 crop`
- `cargo test -p pdfcraft-automation -j 2 crop`
- `cargo test -p pdfcraft-ui-egui -j 2 crop`

## Definition of done

- Export is driven from the persisted crop-review dataset and uses the shared package writer.
- OCR of the selected portion follows the already-established export/OCR rule and is surfaced clearly.
- Desktop and automation stay aligned where they overlap.
- The repo documents the shipped crop-review workspace clearly enough for future contributors to extend it without collapsing back into the snapshot tool.

## Assumes

Sprint 6 added the project-folder persistence lifecycle for the crop-review workspace.
