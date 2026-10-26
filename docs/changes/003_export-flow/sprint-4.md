# Sprint 4 — Desktop rectangle export flow

## Goal

Add the desktop app flow for creating a rectangle, collecting typed metadata, and exporting the package through the same engine path that headless callers already use.

## Files

- `crates/ui-egui/src/zoom_snap.rs`
- `crates/ui-egui/src/commands.rs`
- `crates/ui-egui/src/files.rs` or the nearest export/save dialog helper
- the nearest UI module that owns rectangle-selection state and metadata entry
- `crates/ui-egui/tests/it/export_flow.rs`
- any tiny shared wiring needed in engine-facing call paths

## Reuse

- Reuse the headless/engine flow from Sprint 3; do not fork export logic in the UI.
- Reuse any existing rectangle-selection interaction already present in the app rather than inventing a second drag model.
- Reuse existing file dialog, progress, and error-notification patterns from other export flows.
- Reuse the minimal LaTeX/package writer behavior already implemented below the UI layer.

## Tests

- UI integration test covering creation of one export rectangle, typed metadata entry, and successful export.
- UI integration test covering multiple rectangle items if the desktop flow supports them in v1; otherwise one clear refusal/path test proving the v1 limit.
- UI integration test covering user-visible failure when the rectangle or metadata is invalid.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-ui-egui -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-ui-egui -j 2 export`

## Definition of done

- A desktop user can create the rectangle that drives export, type the required metadata, and export the standard package.
- The desktop flow uses the same engine export request shape as automation.
- Success and failure are surfaced clearly to the user.
- The UI still presents the feature as a focused export flow, not as a generic report authoring surface.

## Assumes

Sprint 3 exposed the rectangle-driven export flow headlessly and verified it end to end.
