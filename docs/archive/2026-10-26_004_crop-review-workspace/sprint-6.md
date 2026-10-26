# Sprint 6 — Project-folder persistence lifecycle

## Goal

Persist the crop-review workspace as a self-contained project folder with UTF-8 JSON metadata plus copied `source.pdf`, and reload it back into the workspace faithfully.

## Files

- the nearest engine/shared modules that should own crop-review project persistence
- the UI or file-handling modules that should expose save/open lifecycle for crop-review projects
- focused persistence tests and at least one end-to-end UI or engine reload test

## Reuse

- Reuse existing file chooser, atomic writing, and path handling patterns from the rest of the app.
- Reuse the crop model/store from Sprint 2.
- Reuse the shell's existing open/reload flows where appropriate, but do not force the crop-review data into autosave/recovery internals if it is not a natural fit.

## Tests

- Persistence test for UTF-8 JSON with non-ASCII metadata preserved directly.
- Persistence test for copied `source.pdf` and project reload reconstruction.
- Reload test proving `project.json -> crop store -> right-side panels + overlay` is faithful.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-engine -j 2 --all-targets -- -D warnings`
- `cargo clippy -p pdfcraft-ui-egui -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-engine -j 2 crop`
- `cargo test -p pdfcraft-ui-egui -j 2 crop`

## Definition of done

- The crop-review workspace can be saved and reopened from a self-contained project folder.
- Crop geometry, identifiers, metadata, OCR fields, and timestamps survive the round trip.
- The JSON source of truth is UTF-8 and readable without ASCII escaping.

## Assumes

Sprint 5 introduced the dedicated crop-creation tool and in-memory persistent crop store usage in the desktop workflow.
