# Sprint 3 — Headless rectangle export flow

## Goal

Expose rectangle-driven evidence export through automation/CLI so the feature is usable headlessly before the desktop UI is wired, with the same rectangle semantics and metadata contract defined in Sprint 1 and modeled in Sprint 2.

## Files

- `crates/automation/src/lib.rs`
- `crates/automation/src/tools.rs`
- the nearest automation export tool module for the new rectangle-driven surface
- `crates/automation/tests/it/automation.rs`
- any tiny engine-side helper adjustments needed by the tool surface

## Reuse

- Reuse the engine export request/model from Sprint 2.
- Reuse existing automation geometry conventions: 1-based pages and displayed top-left-origin rectangles in points.
- Reuse existing path-confinement and package-writing behavior from current automation export/save tools.
- Reuse existing JSON-schema and tool-reporting patterns so the feature is reachable through `pdfcraft-cli run` without a separate one-off CLI system.

## Tests

- End-to-end automation test exporting one rectangle-backed evidence item with typed metadata.
- End-to-end automation test exporting multiple rectangle-backed evidence items in deterministic order.
- Failure-path automation test for invalid pages, invalid rectangles, or invalid output paths.
- Contract test that the tool schema clearly describes rectangle coordinates and metadata fields.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-automation -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-automation -j 2`

## Definition of done

- The feature is reachable headlessly through one automation/CLI surface.
- A caller can supply rectangle-backed evidence input plus typed metadata and receive the standard export package.
- The tool routes through the shared engine path rather than duplicating export logic in automation code.
- The shipped behavior remains snapshot-only and does not claim generic reporting or non-rectangle evidence support.

## Assumes

Sprint 2 introduced the engine export model for rectangle-driven requests and kept the package writer compatible with that model.
