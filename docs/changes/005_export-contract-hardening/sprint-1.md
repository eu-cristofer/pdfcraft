# Sprint 1 — Document export contract 1.1 and AI artifacts

## Goal

Update the committed docs so they reflect the current shipped package behavior: `manifest_version 1.1`, canonical per-item `ocr_text`, derived `report.marker.json`, optional `report.docx`, and the current minimal `report.tex` structure.

## Files

- `docs/snapshot-export-spec.md`
- `docs/rectangle-driven-export.md`
- `CHANGELOG.md`

## Reuse

- Reuse the current engine implementation in `crates/engine/src/export.rs` as the executable source of truth.
- Reuse the package schema language already present in `docs/snapshot-export-spec.md` rather than inventing a parallel document.

## Tests

- No new code tests required if this sprint stays documentation-only.
- Manually confirm every documented artifact/field against the current engine output shape.

## Acceptance

- `cargo fmt --check`
- Review `docs/snapshot-export-spec.md` and `docs/rectangle-driven-export.md` for consistency with `crates/engine/src/export.rs`

## Definition of done

- The export spec names `manifest_version 1.1`.
- The manifest item schema documents optional `ocr_text`.
- The package contents list documents `report.marker.json` and optional `report.docx` as derived artifacts.
- The LaTeX section describes the current minimal `enumerate` + `minipage` structure rather than the older figure-only shape.
- The changelog records the contract expansion clearly.

## Assumes

Assumes the current engine export implementation already emits the 1.1 manifest, `report.marker.json`, and optional `report.docx`.
