# Sprint 1 — Workspace contract and product boundaries

## Goal

Lock the repo-local source of truth for the persistent crop-review workspace: what the workspace is, how it differs from the original snapshot tool, what the Crops and Metadata right-side panels must do, what fields a crop item carries, how ordering works, and what project persistence must preserve.

## Files

- one new repo-local contract note under `docs/` for the crop-review workspace
- `docs/rectangle-driven-export.md` only if it needs a narrow cross-reference to the new workspace direction
- `docs/changes/004_crop-review-workspace/MOTIVATION.md`
- `docs/changes/004_crop-review-workspace/README.md` only if wording needs adjustment after the contract note is written
- `CHANGELOG.md` if the contract note is user-visible enough to register

## Reuse

- Reuse the existing right-panel vocabulary and shell patterns already present in `crates/ui-egui` instead of inventing a second UI architecture.
- Reuse the rectangle/export/OCR package boundaries already documented in `docs/rectangle-driven-export.md` and `docs/snapshot-export-spec.md`.
- Reuse the product distinction already established in conversation: snapshot stays copy-only; persistent crop review is separate.

## Tests

- No new product code is required unless a tiny clarifying test is the smallest way to verify a current shell or ordering assumption.

## Acceptance

- `cargo fmt --check`

## Definition of done

- The repo contains one concrete source-of-truth note for:
  - the crop-review workspace purpose and boundaries
  - Crops and Metadata right-side panel behavior
  - crop ordering and selection synchronization rules
  - the persisted crop item fields and geometry rule
  - the project-folder persistence shape
  - the relationship between crop review and later export
- Later sprints no longer depend on the planning conversation to know whether crop review is separate from snapshot copy.

## Assumes

This is the first sprint.
