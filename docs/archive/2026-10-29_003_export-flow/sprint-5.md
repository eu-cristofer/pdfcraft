# Sprint 5 — OCR integration and final hardening

## Goal

Add OCR of the selected portion into the rectangle-driven export flow, then harden tests, docs, and user-facing behavior so the shipped v1 feature is stable and clearly bounded.

## Files

- OCR-related engine/export integration points
- automation and UI call sites that need to surface OCR-backed export results
- export-related engine, automation, and UI tests
- final docs updates in `docs/` and any relevant crate README
- `CHANGELOG.md`
- `README.md` only if the user-facing feature should be discoverable there now

## Reuse

- Reuse the rectangle-driven flow implemented in Sprints 2–4.
- Reuse the existing OCR pipeline/models already used elsewhere in the repo, rather than introducing a second OCR stack for export.
- Reuse the minimal export package and LaTeX structure already established earlier in this project.
- Reuse existing contributor-facing docs style and test naming conventions.

## Tests

- Coverage proving OCR of the selected portion is incorporated according to the Sprint 1 contract.
- Coverage for deterministic package output across repeated runs with the same rectangle and metadata inputs.
- Coverage for rotated pages and displayed-rectangle semantics through the full OCR-backed export flow.
- Coverage for OCR failure or unavailable-model behavior according to the chosen v1 rule.
- End-to-end automation and desktop tests proving the shipped flow stays aligned across both entrypoints.

## Acceptance

- `cargo fmt --check`
- `cargo clippy -p pdfcraft-engine -j 2 --all-targets -- -D warnings`
- `cargo clippy -p pdfcraft-automation -j 2 --all-targets -- -D warnings`
- `cargo clippy -p pdfcraft-ui-egui -j 2 --all-targets -- -D warnings`
- `cargo test -p pdfcraft-engine -j 2 export`
- `cargo test -p pdfcraft-automation -j 2 export`
- `cargo test -p pdfcraft-ui-egui -j 2 export`

## Definition of done

- OCR of the selected portion is part of the export flow in the way the contract promised, not an undocumented side channel.
- App and automation/CLI both drive the same shipped feature surface.
- Tests protect the core guarantees: rectangle semantics, typed metadata mapping, OCR inclusion, deterministic package output, and minimal LaTeX generation.
- The repo documents the shipped v1 behavior clearly enough for future contributors to extend it without broadening it into a generic report system.

## Assumes

Sprint 4 wired the rectangle-driven export flow into the desktop UI on top of the shared headless and engine path.
