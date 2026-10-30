# Sprint 1 — Product contract and source mapping

## Goal

Write the repo-local contract for selected-rectangle export: exactly which annotation kinds are supported in v1, how their fields map to export `caption`/`note`, which rectangle semantics are exported, and how selection is passed from UI/automation into the engine.

## Files

- one new repo-local decision/spec note under `docs/` for selected-rectangle export
- `docs/changes/002_selected-rec/MOTIVATION.md`
- `docs/changes/002_selected-rec/README.md` only if wording needs adjustment after the discovery note is written
- `CHANGELOG.md` if the new decision note is user-visible enough to register

## Reuse

- Reuse `docs/snapshot-export-spec.md` as the package contract that remains unchanged.
- Reuse the current comments model and selection surfaces in:
  - `crates/ui-egui/src/comments.rs`
  - `crates/ui-egui/src/comments_panel.rs`
  - `crates/automation/src/comments.rs`
  - `crates/engine/src/lib.rs`
- Reuse the earlier product analysis already discussed for `pdf_handler`; do not restate it loosely from memory.

## Tests

- No new product code required yet unless a tiny clarifying test is the smallest way to verify a current assumption.

## Acceptance

- `cargo fmt --check`

## Definition of done

- The repo contains a concrete source-of-truth note for:
  - supported annotation kinds in v1
  - mapping from annotation data to export `caption` and `note`
  - whether export uses shown/displayed rectangle semantics or raw annotation rects
  - whether export order comes from current selection order or canonical document order
  - headless and desktop entrypoints to be built in later sprints
- The later sprints no longer depend on the planning conversation to know what to implement.

## Assumes

This is the first sprint.
