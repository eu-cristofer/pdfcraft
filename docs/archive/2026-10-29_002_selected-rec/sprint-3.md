# Sprint 3 — Headless export tool and automation parity

## Goal

Expose selected-rectangle snapshot export through automation/CLI so the feature is reachable headlessly before the desktop UI is wired.

## Files

- `crates/automation/src/comments.rs` and/or the nearest export tool module
- `crates/automation/src/lib.rs`
- `crates/automation/src/tools.rs`
- `crates/automation/tests/it/automation.rs`
- any tiny engine-side helper adjustments needed by the tool surface

## Reuse

- Reuse the engine bridge from Sprint 2.
- Reuse existing comment selection/addressing conventions (`page` + `index`, or existing comment id rules) from `comment_list` / `comment_edit`.
- Reuse existing output writing and delivery patterns from automation export/save tools.

## Tests

- End-to-end automation test exporting one selected rectangle comment.
- End-to-end automation test exporting multiple selected rectangle comments.
- Failure-path automation test for unsupported or missing comment targets.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-automation -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-automation -j 2`

## Definition of done

- The feature is reachable headlessly through one automation tool.
- The tool writes a valid export package using the existing snapshot export backend.
- The output contract is documented in the tool schema and exercised end to end.

## Assumes

Sprint 2 introduced an engine-level bridge from selected rectangle comments to normalized export items.
