# 001_snapshot-export-contract — Motivation

## Problem

The current snapshot export prototype is a good MVP direction, but it is underspecified in the places that most affect correctness and long-term stability: coordinate semantics, deterministic ordering, file naming, source-PDF semantics, rendering settings, and the boundary between logical evidence data and LaTeX presentation. If implemented directly from the prototype, the feature risks producing exports that drift across refactors, mismatch between `manifest.json` and `.tex`, or become difficult to extend to additional evidence kinds and output formats.

## Approach

The project will promote the export manifest to the canonical contract and make the package deterministic end to end. The exporter will produce a self-contained package with a versioned manifest, rendered snapshot images, the exact source PDF used for rendering, and LaTeX generated from the same ordered item list. The plan keeps the implementation scoped to snapshot evidence for v1 while deliberately reserving room for future expansion to other evidence kinds and non-LaTeX consumers.

## Key decisions

- 2026-10-25 — Use a versioned export contract with `manifest.json` as the canonical source; LaTeX becomes a derived artifact.
- 2026-10-25 — Keep v1 snapshot-only, but model each item with `kind = "snapshot"` so future kinds can be added without breaking the format.
- 2026-10-25 — Prefer deterministic ordering, deterministic `item_N` IDs, and deterministic image filenames derived from IDs rather than captions.
- 2026-10-25 — Harden the spec before broad implementation: coordinate contract, source-PDF semantics, rendering defaults, validation rules, and text/LaTeX field semantics must be explicit.
- 2026-10-25 — Optimize for reproducibility and future expansion over the smallest possible first implementation.

## Risks

- Coordinate mismatches between stored snapshot rectangles, renderer expectations, and displayed page rotation can silently produce wrong crops.
- If manifest order, IDs, LaTeX labels, and file names are not all derived from one shared ordered list, outputs may drift.
- Mixed plain text and trusted LaTeX content can cause broken report output or lock future consumers into LaTeX-specific parsing.
- Exporting a copied `source.pdf` without defining whether it reflects unsaved edits can make the package non-reproducible.
- Overgeneralizing too early could slow delivery; undergeneralizing could force a format rewrite when highlight/text evidence is added later.
