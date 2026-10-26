# Rectangle-Driven Export

Last reviewed: 2026-10-26

## Status

This document is the repo-local source of truth for Sprint 1 of `003_export-flow`.

It defines the product contract for rectangle-driven snapshot export: the engine-boundary rectangle model, the user-typed metadata model for v1, where OCR belongs in the package, the minimal LaTeX artifact that stays in scope, and which older selected-comment assumptions are superseded.

## Scope

This note is intentionally narrow.

It covers:

- the rectangle input model at the engine boundary
- the v1 user-typed metadata model
- the OCR placement decision for v1
- the exact minimal `report.tex` structure that remains in scope
- which earlier selected-comment assumptions no longer apply
- the planned later headless and desktop entrypoints

This note does not define:

- a generic report system
- non-snapshot evidence kinds
- a new package family separate from `docs/snapshot-export-spec.md`
- arbitrary trusted-LaTeX fields
- a requirement that the export rectangle be persisted as a PDF annotation
- final command names or UI copy for later sprints

## Relationship to the package contract

`docs/snapshot-export-spec.md` remains authoritative for:

- package shape
- `manifest.json` canonicality
- deterministic IDs and paths
- coordinate language
- `source.pdf` semantics
- figure-oriented `report.tex` derivation

This note defines how the rectangle-driven product flow feeds that existing package contract.

v1 rectangle-driven export deliberately reuses the existing manifest item text fields:

- the user's typed title or label maps to manifest `caption`
- the user's typed explanatory text maps to manifest `note`

This note does not broaden the package into a new report schema.

## Product contract

### 1. The evidence source is a user-created rectangle

For `003_export-flow`, exported evidence is driven by a rectangle the user creates on a page.

The rectangle is the public feature input.

It is not:

- a selected comment annotation
- an annotation-derived rectangle
- a comment page/index target
- a temporary screen-pixel crop contract

The rectangle may later be created from desktop UI interaction or supplied by a headless caller, but both paths must normalize to the same engine-boundary model.

### 2. Reuse the repo's external rectangle language

The engine boundary MUST reuse the repo's existing external geometry language from automation.

The rectangle contract is therefore:

- `page`: 1-based page number
- `rect.x`: top-left-origin horizontal position in displayed-page points
- `rect.y`: top-left-origin vertical position in displayed-page points
- `rect.width`: width in displayed-page points
- `rect.height`: height in displayed-page points

The fixed coordinate meaning is:

- page origin: top-left
- page space: display
- units: pt
- page rotation: already applied

Implications:

- the rectangle matches what the user selected on the displayed page
- callers do not supply raw PDF user-space coordinates
- callers do not supply transient zoom-space pixels
- later implementation must normalize through one shared engine path before crop rendering and manifest writing

The rectangle MUST satisfy the same basic validity rules already used by snapshot export items:

- `x` and `y` are finite and non-negative
- `width` and `height` are finite and positive

### 3. Canonical engine-boundary item shape

The rectangle-driven engine boundary for one evidence item is conceptually:

```json
{
  "page": 1,
  "rect": {
    "x": 72.0,
    "y": 96.0,
    "width": 216.0,
    "height": 120.0
  },
  "caption": "Short plain-text caption",
  "note": "Optional plain-text note."
}
```

Field meaning:

- `page`: REQUIRED. 1-based page number in displayed-page space.
- `rect`: REQUIRED. The selected rectangle in the shared coordinate contract above.
- `caption`: REQUIRED. Plain-text user-typed metadata.
- `note`: OPTIONAL. Plain-text user-typed metadata.

The primary v1 desktop flow is expected to export one rectangle at a time.

If a later sprint adds a batched path, the engine boundary should accept one explicit ordered list of these item inputs and derive package order, IDs, image filenames, and LaTeX figure order from that list only.

### 4. v1 typed metadata model is explicit and minimal

v1 does not infer metadata from annotations.

The user types metadata explicitly.

The v1 model is deliberately minimal:

- `caption`: required plain-text field
- `note`: optional plain-text field

Rules:

- `caption` is user-authored text, not an annotation type label
- `note` is user-authored text, not annotation contents
- both fields are plain text
- neither field is trusted LaTeX
- later exporters must escape both fields before emitting LaTeX
- v1 does not define additional user-typed fields such as author, tags, category, severity, references, or rich text

This keeps the product aligned with the current snapshot export package instead of creating a broader report schema.

### 5. OCR belongs to the selected portion itself

OCR in this project belongs to the same selected portion as the evidence rectangle.

The OCR source is:

- the same page as the exported item
- the same displayed-page rectangle as the exported crop
- the same exported document state

OCR is not:

- whole-page OCR attached loosely to the item
- comment text reused as OCR
- a second independently chosen rectangle
- a generic transcript system for the entire package

### 6. OCR placement in the package for v1

v1 keeps `manifest.json` canonical and unchanged for rectangle metadata.

For v1, OCR text is derived into `report.tex` from the selected portion.

It is not introduced as:

- a new canonical manifest field
- a sidecar `ocr/` directory
- per-item text files beside the PNGs
- a generic package-wide text database

This is deliberate scope control for Sprint 1:

- the package contract remains anchored on the existing manifest and figure-oriented report structure
- OCR is part of the shipped rectangle-driven flow, but in v1 it remains a derived reporting layer rather than a new canonical manifest schema

If a later project needs canonical machine-readable OCR data in the package, that must be a separate explicit versioned contract change.

### 7. Minimal LaTeX structure that remains in scope

`report.tex` stays minimal and figure-oriented.

The existing snapshot export structure remains the starting point.

The allowed v1 shape is:

- an optional top-level heading such as `\section*{Snapshot Evidence}`
- one figure block per manifest item, in manifest order
- for each figure:
  - `\centering`
  - `\includegraphics[width=\linewidth]{<image.path>}`
  - `\caption{<escaped manifest caption>}`
  - `\label{fig:<id>}`
- if `note` is present, one plain-text `\paragraph{Note.}` block after that figure
- if OCR text is present, one plain-text `\paragraph{OCR.}` block after that figure's note block, or directly after the figure when no note exists

Required rules:

- figure order MUST match `manifest.items`
- labels MUST remain derived from item IDs only
- note text MUST come from the manifest `note` field only
- OCR text MUST come from OCR of the same selected portion only
- all user-authored and OCR text MUST be emitted as escaped plain text

### 8. What remains out of scope for LaTeX

This project does not create a generic report system.

`report.tex` v1 MUST NOT grow into:

- arbitrary user-authored LaTeX injection
- freeform sections per item
- custom figure templates per evidence type
- tables of metadata
- bibliography or citation machinery
- synthesized summaries across items
- a generic narrative report builder
- a second source of truth separate from `manifest.json`

The downstream use case remains a minimal evidence package centered on snapshot figures.

## Superseded earlier assumptions

`docs/selected-rectangle-export.md` remains a historical contract for `002_selected-rec`.

For `003_export-flow`, the following earlier assumptions are superseded:

- selected comments are the feature input
- annotation kind determines exported `caption`
- annotation contents determine exported `note`
- comment page/index addressing is the public feature boundary
- supported-comment-kind filtering defines the product scope

For this project, the active direction is:

- rectangle-driven
- typed metadata is explicit
- OCR belongs to the selected portion
- the package remains snapshot-only and minimal
- the LaTeX artifact remains figure-oriented rather than becoming a generic report layer

## Planned entrypoints for later sprints

Later sprints should build exactly two user-facing entrypoint families, both routed through the engine.

### Headless

One automation/CLI surface under the existing headless stack should accept rectangle-driven export input:

- page
- rectangle
- typed `caption`
- optional typed `note`
- export destination

That surface should live in the usual automation/CLI path and route through the engine rather than bypassing it.

### Desktop

One desktop UI flow should let the user:

- create the rectangle on a page
- type the required metadata
- trigger export through the engine

The desktop flow is rectangle-driven, not comment-selection-driven.

### Engine

The engine owns the normalization seam.

Both headless and desktop callers should converge on one shared engine path that:

- validates page and rectangle input
- maps typed metadata into manifest `caption` and `note`
- reuses the snapshot export package writer
- derives the minimal `report.tex`
- later derives OCR text from the selected portion into that report output

## Summary

The v1 rectangle-driven export contract is intentionally strict:

- the public feature input is a user-created rectangle
- the rectangle reuses the repo's automation-facing displayed-page geometry language
- typed metadata is explicit and maps only to `caption` and optional `note`
- OCR belongs to the selected portion itself
- OCR is derived into `report.tex` in v1 rather than added as new canonical manifest data
- `report.tex` stays minimal, figure-oriented, and non-generic
- desktop and headless flows must route through the same engine contract

This gives later sprints one clear source of truth without depending on the planning conversation.

## Revision history

| Date | Change |
| --- | --- |
| 2026-10-26 | Initial rectangle-driven export contract for `003_export-flow`, including typed metadata, OCR placement, minimal LaTeX scope, and superseded selected-comment assumptions. |
