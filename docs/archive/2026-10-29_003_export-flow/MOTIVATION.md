# 003_export-flow — Motivation

## Problem

PdfCraft now has snapshot export package plumbing in `crates/engine/src/export.rs`, but the user-facing workflow is still not the right one. The earlier `002_selected-rec` project narrowed the problem around selected rectangle comments, yet the clarified product direction is different: export must be driven by a rectangle the user creates, paired with typed metadata, and enriched with OCR from that selected portion. Without that end-to-end flow, the backend package writer is still not the feature the user actually needs.

## Approach

This project keeps the existing snapshot export package family and deliberately stays narrow. v1 is still snapshot-only and still ships the minimal package structure: `manifest.json`, `source.pdf`, cropped evidence images, and a minimal LaTeX artifact. The work here is to define and implement a rectangle-driven export flow that reuses the existing export package machinery, carries user-typed metadata in a stable model, adds OCR of the selected region, and exposes the same capability both headlessly and in the desktop app.

Where this project conflicts with the earlier selected-comment wording, this project supersedes it. The exported evidence source is a user-created rectangle, not a selected comment annotation. Existing snapshot export structures remain valuable, but they must be aligned to the clarified flow rather than treated as a generic report system.

## Key decisions

- 2026-10-26 — The primary evidence source is a user-created rectangle on a page, not a previously created comment annotation.
- 2026-10-26 — User-typed metadata must be modeled explicitly for export rather than inferred from comment kind names or comment bodies.
- 2026-10-26 — OCR belongs to the selected portion itself and is part of the shipped export flow, but it lands after the rectangle and metadata flow is already wired.
- 2026-10-26 — The LaTeX artifact stays minimal and figure-oriented, reusing the existing export structure instead of expanding into a generic report format.
- 2026-10-26 — Headless parity is required: automation/CLI and desktop must route through the same engine export path.
- 2026-10-26 — v1 remains snapshot-only and intentionally does not broaden into a multi-kind evidence or general publishing system.

## Risks

- Rectangle geometry must stay in one clear, display-space contract across app, automation, engine normalization, manifest output, and raster cropping.
- If metadata is bolted on ad hoc in UI code, automation and desktop will diverge quickly.
- OCR integration can easily blur the boundary between required export data and optional diagnostics unless Sprint 1 locks the contract first.
- Reusing the existing `report.tex` path without defining the minimal allowed structure could accidentally grow into a generic report builder.
- The earlier selected-comment project already established some wording and assumptions, so this project must be explicit about which direction now wins when the two plans differ.
