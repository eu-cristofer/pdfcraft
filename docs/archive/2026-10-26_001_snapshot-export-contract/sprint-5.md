# Sprint 5 — Raster/source export and validation

## Goal

Finish the export package writer: emit the exact source PDF, render the snapshot crops, write the package atomically or near-atomically, and validate that all referenced artifacts exist and match the manifest.

## Files

- export package writer implementation
- snapshot crop rendering/export code paths
- validation/finalization helpers
- tests for package completeness and failure behavior

## Reuse

- Reuse the renderer/export path already used for snapshot images where possible.
- Reuse existing document save/serialization logic for `source.pdf` semantics.
- Reuse existing atomic-write helpers and filesystem error handling if the crate already has them.

## Tests

- Integration test for one-item package export.
- Integration test for multi-item export across pages.
- Test for rotated-page cropping behavior.
- Failure-path test for invalid rect or write failure leaving no misleading final package.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p <relevant-crate> -j 2 --all-targets -- -D warnings`
- `cargo test -p <relevant-crate> -j 2`

## Definition of done

- The exported package contains `manifest.json`, `source.pdf`, rendered images, and `.tex`.
- `source.pdf` matches the contract defined in Sprint 2.
- All manifest file references resolve inside the package.
- Rendering/write failures are surfaced as actionable errors and do not leave a false-success package behind.

## Assumes

Sprint 4 can already produce manifest and LaTeX outputs from normalized items.
