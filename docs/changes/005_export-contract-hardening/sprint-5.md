# Sprint 5 — UI, automation, and docs parity sweep

## Goal

Do one final parity pass across docs, automation schemas, desktop export behavior, and package outputs so every surface reflects the same contract.

## Files

- `crates/automation/src/tools.rs`
- `crates/automation/tests/it/automation.rs`
- `crates/ui-egui/src/export_ui.rs`
- `crates/ui-egui/tests/it/export.rs`
- docs touched in earlier sprints, if any wording drift remains

## Reuse

- Reuse the shared package writer and the already-updated docs.
- Reuse existing automation schema tests and UI export tests.

## Tests

- Focused automation tests for package outputs with OCR/marker/docx enabled
- Focused UI export tests for destination folder, package folder name, and report.docx option

## Acceptance

- `cargo fmt --check`
- `cargo check -p pdfcraft-engine -p pdfcraft-automation -p pdfcraft-ui-egui -j 2`
- `cargo test -p pdfcraft-engine -j 2 export -- --nocapture`
- `cargo test -p pdfcraft-automation -j 2 crop_review_package_export -- --nocapture`

## Definition of done

- Docs, engine, automation, and desktop export behavior tell the same story.
- The AI-oriented and Word-oriented outputs are discoverable and schema-documented where appropriate.
- Remaining risks, if any, are recorded explicitly rather than left implicit in code.

## Assumes

Assumes Sprints 1–4 are complete so this sprint can validate parity instead of redefining behavior.
