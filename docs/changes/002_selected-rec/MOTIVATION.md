# 002_selected-rec — Motivation

## Problem

PdfCraft now has a snapshot export v1 backend, but the workflow a user actually wants is still missing: open a PDF, create or select rectangle comments over report items, and export those selections as a standalone package with `manifest.json`, `source.pdf`, cropped images, and `report.tex`. Without a user-facing or headless entrypoint, the backend is not yet usable in practice.

## Approach

This project will keep the current standalone snapshot export v1 package format and add a thin bridge from existing rectangle comments to that package. The first shipped scope is intentionally narrow: selected rectangle comments only, using existing annotation geometry, current deterministic export ordering, and the existing simpler `report.tex` format. The feature will be exposed headlessly first for parity, then through the desktop UI.

## Key decisions

- 2026-10-26 — Use the current simpler standalone `report.tex` format rather than matching the older `pdf_handler` LaTeX block template.
- 2026-10-26 — First shipped source of evidence is selected rectangle comments, not arbitrary drag regions.
- 2026-10-26 — Prefer automation/CLI parity before desktop-only wiring, per repo rules.
- 2026-10-26 — Keep v1 snapshot-only; do not add highlight/text-only evidence in this project.
- 2026-10-26 — Reuse existing annotation selection and comment infrastructure rather than inventing a separate export-only rectangle model.

## Risks

- Rectangle comments in PdfCraft do not carry the same structured caption/comment split as the earlier `pdf_handler` app, so the mapping from comment data to `caption` and `note` must be explicit and conservative.
- The app and automation may disagree about selected comments unless both route through one engine-level mapping function.
- Annotation rectangles may need shown-rect rather than raw PDF rect semantics; choosing the wrong one would export the wrong crop.
- UI-only implementation would violate the repo's headless parity rule.
- Over-broad support for many annotation kinds in the first pass could produce confusing or unstable export behavior.
