# Sprint 4 — Formalize crop-review mutations at the engine boundary

## Goal

Move crop-review add/update/delete operations toward explicit engine-owned operations instead of direct store mutation from UI and automation code.

## Files

- `crates/engine/src/lib.rs`
- `crates/ui-egui/src/*` where crop review mutates the store directly
- `crates/automation/src/*` where crop review mutates the store directly

## Reuse

- Reuse the existing crop-review store and canonical ordering logic.
- Reuse the engine's document/save/refresh lifecycle rather than inventing a side path.

## Tests

- Focused engine/UI/automation regression coverage that crop add/update/delete still work.
- Coverage that the document's save lifecycle persists embedded crop review after those operations.

## Acceptance

- `cargo fmt --check`
- `cargo check -p pdfcraft-engine -p pdfcraft-automation -p pdfcraft-ui-egui -j 2`
- Focused crate tests for crop review and export paths

## Definition of done

- UI and automation no longer rely on naked `crop_review_mut().upsert/remove` for routine crop review changes where an engine operation is more appropriate.
- Crop review becomes more obviously part of the engine-owned document lifecycle.
- The resulting behavior stays parity-safe between desktop and headless paths.

## Assumes

Assumes the persistence invariants from Sprint 3 are already covered so this sprint can safely refactor call sites without losing save/reopen behavior.
