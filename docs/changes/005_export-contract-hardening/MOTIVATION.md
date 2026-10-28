# 005_export-contract-hardening — Motivation

## Problem

The crop-review export flow has now moved beyond the original `003_export-flow` v1 contract in three important ways: OCR is no longer just a derived report detail and can be useful as canonical machine-readable package context, the package now has AI-oriented and Word-oriented derived report artifacts, and crop-review state can be embedded into the saved PDF itself for convenience. Those changes are useful, but today they outpace the written contract and leave one architectural seam under-specified: crop-review state is still mutated from UI and automation code more directly than the rest of the engine's edit lifecycle.

## Approach

This project records the evolved contract explicitly and then hardens the implementation to match it. The package contract should stay centered on one canonical `manifest.json`, with `report.tex`, `report.docx`, and `report.marker.json` all derived from the same ordered item list. The crop-review workspace should keep its external `source.pdf + project.json` project-folder shape as the canonical relocatable workspace while documenting embedded PDF persistence as a convenience mirror. Finally, crop-review mutations should move toward explicit engine-owned operations so save, dirty tracking, persistence, automation parity, and future undo/redo semantics stay aligned with the rest of PdfCraft.

## Key decisions

- 2026-10-28 — Canonical package data remains in `manifest.json`; AI and office artifacts stay derived from the same ordered item list.
- 2026-10-28 — OCR can become canonical package context if and only if the contract version is updated explicitly.
- 2026-10-28 — `report.marker.json` is the AI-oriented derived artifact name for now; it is not a second source of truth.
- 2026-10-28 — The external crop-review project folder remains the canonical relocatable workspace shape; embedded PDF persistence is documented as convenience persistence, not a replacement contract.
- 2026-10-28 — The current direct `CropReviewStore` mutations are acceptable as a bridge but should be formalized behind engine-owned operations in a follow-up hardening step.

## Risks

- Leaving the docs on the old contract while the code emits a new package shape creates ambiguity for every later caller.
- Treating `report.marker.json` or `report.docx` as canonical would create multiple sources of truth and contradict the repo's export philosophy.
- Embedded persistence inside the PDF can interact badly with save fidelity and signatures unless its invariants are written down and tested explicitly.
- Crop-review mutations performed outside explicit engine operations may drift from the repo's edit lifecycle expectations (dirty tracking, save semantics, future undo/redo).
- Desktop and automation could diverge again if the new fields and artifacts are not documented and schema-tested together.
