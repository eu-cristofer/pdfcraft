# Sprint 2 — Crop model and ordered store

## Goal

Introduce the persistent crop item model and canonical ordered store used by the workspace, with geometry stored in document/PDF coordinate space and enough fields to back the Crops panel, Metadata panel, persistence, and export.

## Files

- the nearest engine-facing or shared module that should own crop-review data structures
- any UI-facing glue needed to read/write that model without making it UI-local only
- focused tests for canonical crop ordering and field preservation

## Reuse

- Reuse the existing export item semantics where they overlap: page number, rectangle meaning, typed metadata, and OCR fields.
- Reuse existing document/session state patterns instead of inventing a parallel mutable store abstraction unless the code clearly needs one.
- Reuse `chrono`-backed timestamp behavior already used elsewhere in the repo.

## Tests

- Unit tests for stable ordering by `(page_number, rect_y, rect_x)` ascending.
- Unit tests proving crop geometry is stored in document/PDF coordinates, not viewport pixels.
- Unit tests for stable identifier and metadata field round-tripping inside the in-memory model.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-engine -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-engine -j 2 crop`

## Definition of done

- There is one persistent crop item shape with stable id, geometry, kind, metadata, OCR fields, and timestamps.
- The store exposes canonical ordered iteration suitable for the Crops panel and export.
- The implementation does not depend on viewport/screen coordinates as source of truth.

## Assumes

Sprint 1 locked the crop-review workspace contract, field list, ordering rule, and geometry rule.
