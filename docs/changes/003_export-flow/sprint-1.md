# Sprint 1 — Product contract and flow definition

## Goal

Lock the repo-local contract for rectangle-driven evidence export: how a user-created rectangle is represented, which metadata fields the user types, where OCR belongs in the package, and what the minimal LaTeX artifact must and must not do.

## Files

- one new repo-local contract note under `docs/` for rectangle-driven export
- `docs/snapshot-export-spec.md` only if the package contract needs a narrow, explicit extension for rectangle metadata and OCR
- `docs/selected-rectangle-export.md` only if conflicting wording needs an explicit superseding note instead of leaving stale direction in place
- `docs/changes/003_export-flow/MOTIVATION.md`
- `docs/changes/003_export-flow/README.md` only if wording needs adjustment after the contract note is written
- `CHANGELOG.md` if the new contract note is user-visible enough to register

## Reuse

- Reuse the existing package contract and writer behavior already documented in:
  - `docs/snapshot-export-spec.md`
  - `docs/snapshot-export-discovery.md`
  - `crates/engine/src/export.rs`
- Reuse the repo's existing external geometry language from automation rather than inventing a second rectangle convention.
- Reuse the current LaTeX export structure already present in the engine as the starting point; define the minimum needed for the downstream use case instead of a new broad report shape.
- Reuse existing product context from `docs/selected-rectangle-export.md` only where it still matches the clarified rectangle-driven direction.

## Tests

- No new product code is required unless a tiny clarifying test is the smallest way to verify a current engine assumption.

## Acceptance

- `cargo fmt --check`

## Definition of done

- The repo contains one concrete source-of-truth note for:
  - the rectangle input model at the engine boundary
  - the user-typed metadata model for v1
  - where OCR text is stored or derived in v1
  - the exact minimal LaTeX structure that remains in scope
  - which earlier selected-comment assumptions are superseded
  - the planned headless and desktop entrypoints for later sprints
- Later sprints no longer depend on the planning conversation to know whether the feature is comment-driven or rectangle-driven.
- The contract is explicit that this project does not create a generic report system.

## Assumes

This is the first sprint.
