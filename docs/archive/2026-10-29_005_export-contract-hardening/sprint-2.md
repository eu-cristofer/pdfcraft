# Sprint 2 — Document crop-review persistence shapes and invariants

## Goal

Clarify the persistence contract of crop review now that the code supports both an external project folder and an embedded PDF attachment mirror.

## Files

- `docs/crop-review-workspace.md`
- `CHANGELOG.md`

## Reuse

- Reuse the current project-folder contract in `docs/crop-review-workspace.md`.
- Reuse the reserved attachment name and JSON shape implemented in `crates/engine/src/export.rs` and `crates/engine/src/lib.rs`.

## Tests

- No new code tests required if this sprint stays documentation-only.
- Verify the documented reserved attachment name matches the engine constant.

## Acceptance

- `cargo fmt --check`
- Review `docs/crop-review-workspace.md` against `crates/engine/src/export.rs` and `crates/engine/src/lib.rs`

## Definition of done

- The crop-review workspace doc explicitly states whether `source.pdf + project.json` remains the canonical relocatable workspace shape.
- The embedded PDF attachment mirror is documented as implemented behavior.
- The reserved attachment name and the persisted JSON fields are documented.
- The relationship between external project persistence and embedded persistence is unambiguous.

## Assumes

Assumes Sprint 1 has already documented the package-side contract changes so persistence docs can refer to the new export shape consistently.
