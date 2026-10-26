# Sprint 1 — Discovery and contract decisions

## Goal

Turn the planning conversation into repo-local, implementation-usable decisions by locating the real snapshot/reporting code paths and documenting the exact answers the later sprints depend on: coordinate system, ordering source, LaTeX trust model, source-PDF semantics, and current rendering/output behavior.

## Files

- `docs/changes/001_snapshot-export-contract/MOTIVATION.md`
- `docs/changes/001_snapshot-export-contract/sprint-1.md`
- one new repo-local decision note under `docs/` or an existing reporting/prototyping doc near the implementation area
- any small README/changelog pointer updates needed to describe the registered discovery output

## Reuse

- Reuse the existing snapshot/export/reporting implementation rather than restating assumptions from the prototype docs.
- Inspect the current LaTeX/report generator path the prototype referenced (`src/latex/generator.py` in the other project, plus the equivalent code in this repo if present).
- Reuse any existing snapshot render/export path, document serialization path, and report ordering logic already in the codebase.

## Tests

- No large feature tests yet; focus on verifying the discovered behavior against real code.
- Add lightweight doc-facing validation only if the repo already has a convention for it.

## Acceptance

- `cargo fmt --check`

## Definition of done

- The repo contains a concrete, implementation-usable record of the discovery answers for:
  - coordinate system and page-rotation handling
  - current snapshot ordering source and desired export ordering
  - whether report comments/captions are plain text or trusted LaTeX
  - what `source.pdf` must represent in the presence of unsaved edits
  - current crop/image format defaults and whether they should be preserved or changed
- The decision record is specific enough that later sprints do not need the planning conversation.

## Assumes

This is the first sprint.
