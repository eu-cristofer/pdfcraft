# Selected Rectangle Export

Last reviewed: 2026-10-26

## Status

This document remains the repo-local source of truth for Sprint 1 of `002_selected-rec`.

For `003_export-flow`, `docs/rectangle-driven-export.md` is the active source of truth. Where the two notes conflict, the rectangle-driven note supersedes this one. In particular, `003_export-flow` is driven by a user-created rectangle with typed metadata, not by selected comment annotations or annotation-derived text.

It defines how PdfCraft will turn a selected comment annotation into one or more snapshot export items under the unchanged package contract in `docs/snapshot-export-spec.md`.

## Scope

This note is intentionally narrow.

It covers:

- which existing comment annotation kinds are eligible for v1 selected-rectangle export
- how existing annotation data maps into manifest `caption` and `note`
- which rectangle semantics become exported `rect`
- how desktop UI selection and automation comment targets cross the engine boundary
- what ordering rule later sprints must implement if an export operation resolves more than one comment target

This note does not define:

- arbitrary drag-region export
- non-rectangle evidence kinds
- changes to `docs/snapshot-export-spec.md`
- a new user-authored caption model separate from current comment data

## Relationship to the package contract

`docs/snapshot-export-spec.md` remains unchanged and authoritative for package shape, manifest schema, coordinate language, deterministic IDs, and `report.tex` derivation.

This note only defines how selected comments become normalized snapshot items that satisfy that package contract.

## Product contract

### 1. v1 export source is selected comments, not freeform drag rectangles

v1 selected-rectangle export is driven by existing comment selection/addressing surfaces.

The exported evidence source is a resolved comment annotation, not a temporary screen-space drag box.

Implications:

- desktop export later starts from the currently selected comment in per-document comment state
- headless export later starts from existing automation comment addressing
- v1 does not accept arbitrary rectangle coordinates as the public feature input
- v1 does not promise export for comment kinds that do not already present one stable displayed rectangle

This preserves the older `pdf_handler` product intent of exporting PDF-backed rectangular evidence, while staying honest about PdfCraft's current surfaces: PdfCraft already has comment selection and comment addressing, but does not currently expose a separate caption/comment crop model.

### 2. Supported annotation kinds in v1

v1 supports only these annotation kinds:

- `Rectangle`
- `Callout`
- `Text Box`

These are the only supported v1 sources for selected-rectangle export even if `comment_kind` or `comment_type_name` can distinguish additional annotation types elsewhere in the repo.

Any other annotation kind is out of scope for v1 and MUST be rejected by the later implementation before package creation.

Reason for the narrow set:

- the feature being shipped in this project is selected rectangle export
- these supported kinds already fit the product model of one selected comment yielding one exported rectangular snapshot item
- current automation already exposes a displayed `rect` for comments, which gives later sprints one stable external rectangle language to reuse
- PdfCraft does not currently expose the older `pdf_handler` two-field crop model with an explicit user-authored caption plus PB comment, so v1 must map from the current comment model without inventing new persisted fields

### 3. Unsupported kinds in v1

The following are not part of the v1 contract:

- highlights
- underlines
- strikeouts
- squiggles
- sticky-note style text comments without the rectangle/text-box/callout model
- lines and arrows
- polygons, polylines, and ink
- stamps
- attachments
- popup-only or reply-only items
- any future annotation kind other than `Rectangle`, `Callout`, or `Text Box`

Later sprints MAY expand support, but only through a new explicit contract update.

## Field mapping to snapshot export items

Each supported selected comment produces exactly one manifest item with `kind = "snapshot"`.

### `caption`

`caption` is required by `docs/snapshot-export-spec.md`.

For v1 selected-rectangle export, `caption` MUST be derived from the selected annotation kind's existing user-visible type name, using the same naming family already surfaced by engine comment type labeling.

The required mappings are:

- `Rectangle` -> `Rectangle`
- `Callout` -> `Callout`
- `Text Box` -> `Text Box`

Additional rules:

- `caption` is plain text
- `caption` MUST NOT be synthesized from page number, author, or comment contents
- `caption` MUST NOT be treated as trusted LaTeX
- `caption` MUST remain short and type-based because PdfCraft does not currently expose a separate user-authored caption field for this feature

This is the deliberate v1 replacement for the older `pdf_handler` explicit caption field: PdfCraft exports the annotation type label as the caption until a distinct caption model exists.

### `note`

`note` is optional in `docs/snapshot-export-spec.md`.

For v1 selected-rectangle export, `note` MUST be derived from the selected annotation's existing comment contents text.

Rules:

- if the selected annotation has non-empty contents text, export it as `note`
- if the selected annotation has no contents text, or only empty/whitespace contents after the normal text cleanup used by current comment/report paths, omit `note`
- `note` is plain text
- `note` MUST NOT be treated as trusted LaTeX
- later `report.tex` generation must continue to escape it as plain text per `docs/snapshot-export-spec.md`

This keeps v1 aligned with the current repo trust model, where comment/report text is data that must survive plain-text export rather than markup that is trusted by default.

### Fields that do not map into `caption` or `note`

The selected-rectangle v1 contract does not promote any of these into manifest text fields:

- page number
- author
- creation or modification date
- annotation index in `/Annots`
- reply threading
- raw PDF subtype names
- any callout-line geometry
- any other annotation metadata not already needed for item resolution or rectangle extraction

Those fields may still be used internally for resolution, validation, or future UX, but they are not part of the v1 snapshot item text mapping.

## Rectangle semantics

### Exported rectangle is the displayed comment rectangle

For supported kinds, the exported manifest `rect` MUST use the displayed rectangle semantics already aligned with the snapshot export package contract and current automation comment listing.

The exported rectangle is:

- the selected annotation's displayed rectangle
- on the displayed page
- after page rotation has been applied
- top-left-origin
- measured in points
- independent of transient UI zoom

This means the selected-rectangle export note adopts the same external rectangle language already used by the package contract:

- `page_origin = "top-left"`
- `page_space = "display"`
- `units = "pt"`
- `page_rotation = "already-applied"`

### Export does not use raw PDF `/Rect` as the public contract

The v1 exported `rect` is not the raw annotation rectangle in PDF user space.

Internal code may still read raw annotation geometry during resolution, but later implementation must normalize through one shared conversion path before writing the manifest.

The public exported rectangle contract is therefore the displayed rectangle seen by external callers, not the underlying raw PDF coordinate representation.

### Callout and text-box specifics

For `Callout` and `Text Box`, v1 still exports one rectangular snapshot item.

The exported `rect` is the annotation's displayed rectangle only.

Implications:

- any callout leader line geometry is not exported as a separate field
- v1 does not attempt to export a non-rectangular hull
- v1 does not derive multiple items from one selected comment
- the snapshot image is rendered from that one displayed rectangle

### Missing or invalid rectangle handling

If a later implementation cannot resolve one valid displayed rectangle for the selected supported comment, the export MUST fail.

It MUST NOT guess a fallback rectangle from screen pixels, transient widget bounds, or ad hoc per-kind geometry.

## Selection and engine boundary

### Canonical target shape at the engine boundary

Later sprints should use one engine-facing selected-comment target shape based on comment addressing, not on raw rectangles.

The canonical target meaning is:

- `page`: 1-based page number
- `index`: 1-based annotation index within that page's `/Annots` array

This matches the existing automation-facing comment addressing model and avoids leaking UI-local zero-based indexing into the engine boundary.

### Desktop UI handoff

The current desktop selection surface is `CommentView.selected` in `crates/ui-egui/src/comments.rs`, where selection is stored as `Option<(usize, usize)>` with zero-based `(page, index)` semantics.

Later desktop implementation should therefore:

- require one currently selected comment
- convert that zero-based UI selection to the engine-facing 1-based target exactly once at the UI-engine boundary
- invoke the engine with the normalized selected-comment target
- avoid passing raw rectangle coordinates from the UI

The comments panel already exposes comment-level command IDs in `crates/ui-egui/src/comments_panel.rs`, so the later desktop entrypoint should be a comment action wired from that existing command surface rather than a separate rectangle-drag workflow.

### Automation handoff

Automation already exposes comment-focused tooling in `crates/automation/src/comments.rs` with 1-based page/index addressing and a displayed `rect`.

Later headless implementation should therefore:

- reuse existing comment target resolution conventions
- pass the 1-based target directly into the engine without a second addressing scheme
- keep the selected-rectangle feature comment-driven, not freeform-rectangle-driven

This preserves desktop/headless parity around one shared target model.

### Engine resolution responsibilities

Once the engine receives a normalized selected-comment target, later implementation must:

- resolve the exact annotation from the current document state
- verify that its kind is one of the three supported v1 kinds
- derive the exported displayed rectangle through one shared normalization path
- derive `caption` and optional `note` from the current annotation data
- produce one normalized snapshot export item for the package writer

## Ordering contract

### Single-selection desktop export

For the primary desktop path, later sprints are expected to export one selected comment at a time.

In that case, ordering is trivial because the normalized item list contains exactly one item.

### If an operation resolves multiple selected comments

If a later sprint adds a multi-target path for this feature, export order MUST use canonical document order after target resolution, not transient selection order.

The canonical rule is:

- sort by page ascending
- then by annotation index in `/Annots` ascending

The engine MUST use that canonical resolved order to derive:

- `manifest.items`
- positional `item_N` IDs
- `evidence/item_N.png` filenames
- `report.tex` figure order

The engine MUST NOT derive export order from:

- desktop click order
- temporary UI list order
- automation request order
- filesystem order
- comment contents
- annotation type name

This choice keeps the package deterministic and matches the existing addressable comment model.

## Entry points to build in later sprints

This sprint does not implement product code, but later sprints should build these entry points against this contract:

- desktop: a comments-panel action that exports the currently selected supported comment through the engine
- headless: an automation/CLI entrypoint that accepts existing comment targets and routes them through the same engine path
- engine: one shared selected-comment-to-snapshot normalization path used by both desktop and headless callers

The engine boundary is the contract seam. UI and automation choose targets; the engine resolves supported comments into normalized snapshot items; package writing stays governed by `docs/snapshot-export-spec.md`.

## Failure rules specific to this feature

Later implementations of selected-rectangle export MUST fail the operation if any of the following is true:

- no comment is selected or targeted
- the target does not resolve to an annotation in the current document state
- the resolved annotation kind is not `Rectangle`, `Callout`, or `Text Box`
- the resolved annotation does not yield one valid displayed rectangle
- the comment text mapping cannot produce valid plain-text manifest fields under the existing package contract

These failures are in addition to the existing package-level failure rules in `docs/snapshot-export-spec.md`.

## Summary

The v1 selected-rectangle export contract is intentionally strict:

- selected comments are the only source
- supported kinds are only `Rectangle`, `Callout`, and `Text Box`
- `caption` is the annotation type label
- `note` is the annotation contents text when present
- exported `rect` uses displayed-page top-left-origin points after rotation
- desktop and automation both resolve into one engine-facing page/index target model
- any future multi-target export uses canonical document order, not selection order

This gives later sprints one implementation-ready contract without depending on the planning conversation.
