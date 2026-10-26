# Sprint 2 — Spec hardening and examples

## Goal

Rewrite the prototype snapshot export spec into a stable v1 contract that is implementation-ready and future-safe: canonical manifest, deterministic ordering, explicit coordinate/rendering semantics, text/LaTeX field rules, validation rules, and worked examples.

## Files

- the repo doc that will become the export spec (create or update under `docs/`)
- one example package tree and one example manifest/LaTeX snippet in docs if the spec benefits from embedded examples
- `docs/changes/001_snapshot-export-contract/README.md` only if the sprint title or scope needs a wording adjustment

## Reuse

- Reuse the decisions recorded in Sprint 1; do not reopen settled questions unless the code contradicts them.
- Reuse the existing snapshot block structure exactly where the downstream pipeline depends on syntax.
- Reuse existing docs style from `docs/` and the repo’s current Markdown conventions.

## Tests

- If the repo has schema/doc tests for specs, wire into them; otherwise keep verification at file-review level plus formatting.

## Acceptance

- `cargo fmt --check`

## Definition of done

- The export spec defines:
  - package contents
  - canonical manifest schema and versioning
  - ordered item list and deterministic IDs
  - coordinate contract
  - source-PDF semantics
  - image format/render settings contract
  - LaTeX generation contract and trusted/plain text field behavior
  - validation rules and failure behavior
  - explicit future-expansion stance for additional evidence kinds
- The spec contains at least one realistic example manifest and one realistic package layout.

## Assumes

Sprint 1 recorded the real implementation constraints and decisions in repo-local docs.
