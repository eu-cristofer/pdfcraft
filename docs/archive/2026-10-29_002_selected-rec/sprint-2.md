# Sprint 2 — Engine bridge from rectangle comments to snapshot export items

## Goal

Implement the engine-level mapping from selected rectangle comments to normalized snapshot export items, without yet exposing the flow through automation or the desktop UI.

## Files

- `crates/engine/src/lib.rs`
- `crates/engine/src/export.rs`
- focused engine tests for annotation-to-export mapping

## Reuse

- Reuse the snapshot export v1 backend already in `crates/engine/src/export.rs`.
- Reuse current comment/annotation information from `doc.info.annotations` and existing shown-rect semantics where applicable.
- Reuse existing comment type helpers and selection-addressing conventions rather than inventing a second annotation identifier scheme.

## Tests

- Unit tests for supported rectangle comment mapping.
- Unit tests for unsupported comment kinds being refused.
- Unit tests for deterministic item order and page/rect mapping from selected comments.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-engine -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-engine -j 2`

## Definition of done

- The engine can turn explicit selected rectangle comments into normalized snapshot export items.
- The mapping is centralized so UI and automation can reuse it.
- Unsupported evidence/comment kinds fail through the normal error path, not by omission or panic.

## Assumes

Sprint 1 recorded the selected-rectangle product contract and field-mapping decisions.
