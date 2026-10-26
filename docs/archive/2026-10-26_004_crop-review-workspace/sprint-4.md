# Sprint 4 — Metadata panel and selection synchronization

## Goal

Add the right-side `Metadata` panel for the selected crop and wire two-way selection synchronization between overlay, crop list, and metadata, without feedback loops.

## Files

- the nearest UI module that should own metadata editing for crop items
- right-panel dispatch and any selection glue in `crates/ui-egui`
- any tiny shared helper for selecting one crop id without re-emitting loops
- UI integration tests for selection sync and metadata editing

## Reuse

- Reuse existing focused-property-panel patterns from comments, fields, and document properties.
- Reuse existing right-panel state handling and explicit selection update helpers instead of ad hoc callbacks.
- Reuse the crop store model from Sprint 2.

## Tests

- UI test: selecting from the Crops panel selects the overlay item and updates Metadata.
- UI test: selecting from the overlay selects the Crops row and updates Metadata.
- UI test: metadata edits update the selected crop item without oscillating selection state.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-ui-egui -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-ui-egui -j 2 crop`

## Definition of done

- There is one selected-crop path shared by overlay, list, and metadata UI.
- The Metadata panel exposes the selected crop fields clearly enough for later persistence and export.
- Selection updates are stable and do not loop.

## Assumes

Sprint 3 introduced the `Crops` right-side panel and canonical crop row rendering.
