# Sprint 3 — Engine hardening for embedded crop-review persistence

## Goal

Add focused engine tests for the embedded crop-review attachment lifecycle: open, save, reopen, empty-store removal, and no-op behavior on documents with no crop review.

## Files

- `crates/engine/src/lib.rs`
- `crates/engine/src/export.rs`

## Reuse

- Reuse `embedded_crop_review_store`, `sync_embedded_crop_review`, and `EMBEDDED_CROP_REVIEW_ATTACHMENT_NAME`.
- Reuse existing engine test patterns around save/reopen fidelity and attachment handling.

## Tests

- Engine tests covering:
  - opening a PDF with the reserved embedded crop-review attachment restores the store
  - saving and reopening preserves embedded crop-review data
  - saving with an empty crop-review store removes the reserved attachment
  - unchanged documents with no crop-review store do not gain the reserved attachment spuriously

## Acceptance

- `cargo fmt --check`
- `cargo check -p pdfcraft-engine -j 2`
- `cargo test -p pdfcraft-engine -j 2 export -- --nocapture`

## Definition of done

- Engine coverage names the persistence invariants explicitly.
- The embedded persistence path is tested independently of the external project-folder path.
- Signature/save regressions remain green.

## Assumes

Assumes the current embedded persistence code remains in place and documented from Sprints 1 and 2.
