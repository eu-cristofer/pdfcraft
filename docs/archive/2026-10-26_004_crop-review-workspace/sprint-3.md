# Sprint 3 — Crops right-side panel

## Goal

Add the dedicated right-side `Crops` panel with the same overall shell feel as the existing `Comments` panel, rendering one row per stored crop item in canonical order.

## Files

- the right-panel enum / dispatch modules in `crates/ui-egui`
- one new UI module for the crop list panel or row rendering
- any tiny shared view-model glue needed to read the crop store
- `crates/ui-egui/tests/it/*` for crop panel listing behavior

## Reuse

- Reuse the existing right-panel shell and rail behavior already used by `Comments`, `Bookmarks`, and `Pages`.
- Reuse existing list-row rendering patterns where possible, but keep the crop row purpose-specific.
- Reuse the crop store ordering from Sprint 2; do not re-sort ad hoc in the panel.

## Tests

- UI test that the `Crops` panel appears as a right-side panel and lists rows in canonical order.
- UI test that the primary row label is `caption` when present and `Page N` otherwise.
- UI test that the secondary line shows `{Kind} · Page N`.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-ui-egui -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-ui-egui -j 2 crop`

## Definition of done

- The shell exposes a dedicated `Crops` right-side panel.
- The panel lists every crop item in canonical order with the required row semantics.
- The implementation feels like an existing PdfCraft right-side panel rather than a one-off modal list.

## Assumes

Sprint 2 introduced the persistent crop item model and canonical ordered store.
