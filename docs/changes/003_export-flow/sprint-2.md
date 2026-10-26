# Sprint 2 — Engine export model alignment

## Goal

Align the engine export representation with the rectangle-driven flow so the engine can accept user-created rectangles plus typed metadata now, and carry the right shape for OCR integration later without UI- or automation-specific branching.

## Files

- `crates/engine/src/export.rs`
- `crates/engine/src/lib.rs`
- focused engine tests for rectangle-driven export normalization
- any nearby engine module that already owns snapshot export request/response types

## Reuse

- Reuse the snapshot export package writer already in `crates/engine/src/export.rs`.
- Reuse the displayed-page rectangle semantics locked in Sprint 1.
- Reuse the existing minimal `report.tex` generation path instead of adding a separate report generator.
- Reuse existing validation and deterministic ordering helpers wherever the rectangle-driven model can map cleanly onto the current normalized item list.

## Tests

- Unit tests for rectangle request validation.
- Unit tests for typed metadata flowing into normalized export items.
- Unit tests for deterministic ordering and ID/path derivation when more than one rectangle item is exported.
- Unit tests proving the engine model has an explicit OCR slot or equivalent contract seam, even before OCR extraction is wired.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-engine -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-engine -j 2`

## Definition of done

- The engine accepts rectangle-driven export input without depending on desktop-only or automation-only types.
- Typed metadata is represented explicitly in the engine export model rather than inferred from comments.
- The normalized export item path stays deterministic and package-compatible.
- The engine model is ready for OCR integration in Sprint 5 without another shape change that would cascade through app and automation callers.
- The implementation still keeps LaTeX output minimal and figure-oriented.

## Assumes

Sprint 1 recorded the rectangle-driven export contract, metadata model, OCR placement decision, and minimal LaTeX boundaries.
