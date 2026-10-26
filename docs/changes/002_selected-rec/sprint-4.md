# Sprint 4 — Desktop export action for selected rectangle comments

## Goal

Wire the same selected-rectangle export flow into the desktop app through the comments UI, preserving the headless behavior and current standalone `report.tex` output.

## Files

- `crates/ui-egui/src/comments_panel.rs`
- `crates/ui-egui/src/commands.rs`
- `crates/ui-egui/src/files.rs` or the nearest export/save dialog helper
- `crates/ui-egui/tests/it/comments.rs`
- any tiny shared wiring needed in engine/automation-facing code paths

## Reuse

- Reuse the automation/engine flow from Sprint 3; do not fork export logic in the UI.
- Reuse existing comment selection state in `crates/ui-egui/src/comments.rs`.
- Reuse existing file dialog / notification patterns from other export flows.

## Tests

- UI integration test covering selection of at least one rectangle comment and successful export.
- UI integration test covering refusal when the selection contains no exportable rectangle comments.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-ui-egui -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-ui-egui -j 2 comments`

## Definition of done

- A user can export selected rectangle comments from the desktop UI.
- The UI uses the same engine/export logic as automation.
- Success and failure are surfaced clearly to the user.

## Assumes

Sprint 3 exposed the selected-rectangle export flow headlessly and verified it end to end.
