# Sprint 5 — Rectangle creation tool and overlay persistence

## Goal

Introduce a dedicated rectangle-creation tool for crop review, persist created crop items into the store, and keep the original snapshot tool behavior unchanged.

## Files

- the nearest UI modules that own marquee/rectangle gesture handling and overlay drawing
- command wiring for the new crop-review rectangle tool
- any tiny shared glue needed to create crop items from drawn rectangles in document coordinates
- UI tests for creation and original snapshot non-regression

## Reuse

- Reuse the existing marquee/drag mechanics already present in `crates/ui-egui` rather than inventing a second drag system.
- Reuse the crop model/store from Sprint 2 and the right-side panels from Sprints 3–4.
- Preserve the original `edit.snapshot` copy-only behavior.

## Tests

- UI test creating one crop rectangle and confirming it appears in overlay, Crops, and Metadata.
- UI test proving the original snapshot tool still copies pixels and does not open the crop workspace flow.
- UI test proving created rectangles are stored in document/PDF coordinates, not screen-space pixels.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-ui-egui -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-ui-egui -j 2 crop`

## Definition of done

- Users can create persistent crop rectangles through a dedicated tool.
- New crop items are stored durably in memory and rendered back in the overlay.
- The original snapshot tool remains semantically separate.

## Assumes

Sprint 4 wired crop selection and metadata editing around the persistent crop store.
