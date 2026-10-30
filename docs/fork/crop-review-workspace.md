# Crop Review Workspace

Last reviewed: 2026-10-28

## Status

This document is the repo-local source of truth for the crop-review workspace contract.

It defines the product contract for the persistent crop-review workspace: what the workspace is, how it differs from the original snapshot tool, what the `Crops` and `Metadata` right-side panels must do, which fields every crop item carries, how canonical ordering and selection synchronization work, how external project-folder persistence works, how the embedded PDF attachment mirror works, and how this workspace feeds later export.

## Scope

This note is intentionally narrow.

It covers:

- the purpose and boundaries of the persistent crop-review workspace
- the required behavior of the `Crops` and `Metadata` right-side panels
- the single-selection synchronization contract shared by overlay, list, and metadata UI
- the persisted crop item field list
- the crop geometry rule
- the canonical crop ordering rule
- the required on-disk project-folder shape
- the implemented embedded PDF attachment mirror
- how later export consumes stored crop items

This note does not define:

- final command names or final UI copy
- a second UI shell separate from the existing document workspace
- a generic annotation system beyond crop review
- manual drag-reordering of crop items
- a new export package family separate from `docs/fork/snapshot-export-spec.md`
- viewer-only state such as zoom level, scroll position, or panel open/closed state

## Summary

The persistent crop-review workspace is a document-scoped review surface inside PdfCraft's existing document shell.

It is separate from the original snapshot tool:

- `Edit -> Take a snapshot` remains a transient copy action
- the persistent crop-review workspace owns durable crop items, right-side review panels, and persistence
- later export runs from the stored crop dataset, not from a one-off drag gesture

The workspace model is not UI-local source of truth. Desktop UI and later automation must go through `pdfcraft-engine` and the same persisted crop-review data model.

The canonical relocatable workspace shape remains an explicit project folder containing `source.pdf` plus `project.json`.

In addition, the engine also mirrors crop-review metadata into the open PDF as a reserved embedded attachment. That embedded attachment is implemented behavior, but it is not a second relocatable workspace format because it does not carry copied `source.pdf` bytes.

## Purpose And Boundaries

The crop-review workspace exists to let a user build and review a persistent ordered set of crop items for one source PDF before export.

The workspace must provide all of the following:

- one source PDF loaded in the existing document shell
- an overlay that visualizes the stored crop rectangles on the document
- a right-side `Crops` panel listing every stored crop item
- a right-side `Metadata` panel bound to the currently selected crop item
- persistence for the source PDF plus crop-review metadata

The workspace must not do any of the following:

- replace or broaden the original snapshot copy tool
- require a separate standalone screen outside the existing shell
- treat viewport pixels as canonical geometry
- make the UI the only owner of crop-review state
- redefine the snapshot export package contract

The original snapshot tool remains copy-only and transient. It does not create persistent crop items, it does not create a crop-review project folder, and it does not become the entrypoint for the review workflow.

## Right-Side Panels

The workspace uses the existing right-side panel vocabulary and shell patterns already used in `crates/ui-egui`. It does not introduce a second panel architecture.

### Crops Panel

The `Crops` panel is the ordered list view for the stored crop dataset.

It must:

- show exactly one row per persisted crop item
- list rows in canonical crop order
- visually indicate the currently selected crop item
- allow selecting a crop item from the list
- stay synchronized with overlay and metadata selection

Each row must use:

- primary label: `caption` when `caption` is non-empty after trimming, otherwise `Page N`
- secondary label: `{Kind} · Page N`

For v1, `kind` is always `snapshot`, so the displayed kind label is `Snapshot`.

The `Crops` panel is not a manual ordering surface. Users do not drag rows to reorder them. Order is derived from the stored crop item data.

### Metadata Panel

The `Metadata` panel is the detail surface for the currently selected crop item.

It must:

- show an empty state when no crop item is selected
- bind to exactly one selected crop item when a selection exists
- read and write the selected crop item's persisted metadata fields
- stay synchronized with list and overlay selection
- never edit a different crop item than the currently selected one

For v1, the user-editable metadata fields are:

- `caption`
- `note`

The panel may also display persisted non-editable fields for the selected item, including:

- `kind`
- `page_number`
- `rect_pdf`
- `ocr_text`
- `ocr_extracted_at`
- `created_at`
- `updated_at`

The `Metadata` panel does not own ordering. Editing metadata must not create a second ordering source.

## Selection And Ordering

The workspace has one shared selected-crop path.

The canonical selection rules are:

- at most one crop item is selected at a time
- the selection is identified by crop `id`
- selecting from the overlay, the `Crops` panel, or a creation flow updates the same selected `id`
- the overlay highlight, `Crops` row highlight, and `Metadata` panel target must always point at the same selected `id`
- creating a crop item selects the newly created item
- deleting the selected crop item selects the next item in canonical order, or the previous item if there is no next item, or clears selection if the list becomes empty

The canonical ordering rules are:

- crop items are ordered by `(page_number, rect_y, rect_x, id)` ascending
- `page_number` is 1-based
- `rect_y` and `rect_x` are the displayed-page top-left coordinates derived from the stored `rect_pdf` for that page
- `id` is the final deterministic tie-breaker when geometry is identical

This ordering is canonical for:

- the `Crops` panel
- the stored `crops` array in `project.json`
- later export item iteration unless a later contract explicitly defines another ordered view

There is no separate user-managed `sort_index`.

## Crop Item Contract

Every persisted crop item carries the following fields.

| Field | Type | Rule |
| --- | --- | --- |
| `id` | string | Stable per-project identifier. It is preserved across save/load and never reused for another crop item inside the same project. |
| `kind` | string | Crop evidence kind. v1 allows only `"snapshot"`. |
| `page_number` | integer | 1-based PDF page number for the source page that owns the crop rectangle. |
| `rect_pdf` | object | Canonical crop rectangle in PDF/document coordinate space with normalized edges `{ left, bottom, right, top }`. |
| `caption` | string | User-provided short label. May be empty. |
| `note` | string | User-provided explanatory text. May be empty. |
| `ocr_text` | string or null | Stored OCR text for the crop item when available. |
| `ocr_extracted_at` | string or null | RFC 3339 timestamp for when `ocr_text` was last produced. `null` when OCR has not been produced. |
| `created_at` | string | RFC 3339 timestamp for item creation. |
| `updated_at` | string | RFC 3339 timestamp for the most recent persisted mutation to the crop item. |

The geometry rule is strict:

- crop geometry is stored in PDF/document coordinate space, never in viewport pixels
- `rect_pdf` is normalized before persistence so `left < right` and `bottom < top`
- page-relative PDF geometry is the canonical geometry for persistence and later export
- display transforms may derive overlay coordinates and ordering coordinates from `rect_pdf`, but the reverse is not the source of truth once persisted

No additional persisted field is allowed to become a second canonical geometry or ordering source for v1.

## External Project Persistence

A crop-review project folder is the canonical self-contained relocatable workspace for one source PDF and its crop dataset.

The required folder shape is:

- `source.pdf`
- `project.json`

`source.pdf` is a copied local file inside the project folder. The project must not depend on the original absolute source path after save.

`project.json` is UTF-8 JSON and is the canonical persisted metadata file for the crop-review workspace.

Its required top-level shape is:

```json
{
  "schema_version": 1,
  "source_pdf": "source.pdf",
  "crops": []
}
```

The top-level rules are:

- `schema_version` is required and is currently `1`
- `source_pdf` is the relative path string `"source.pdf"`
- `crops` is the canonical ordered array of persisted crop items
- the folder remains relocatable because it stores no required absolute source path
- JSON text is written as UTF-8 and user text remains readable rather than ASCII-escaped where the serializer can preserve direct Unicode

External project persistence must preserve, across save and load:

- crop `id`
- `kind`
- `page_number`
- normalized `rect_pdf`
- `caption`
- `note`
- `ocr_text`
- `ocr_extracted_at`
- `created_at`
- `updated_at`
- canonical item order as defined in this note

External project persistence does not need to preserve transient UI state such as:

- current selection
- current zoom
- current scroll offset
- open or closed panel state

## Embedded PDF Attachment Mirror

The engine also supports embedded crop-review persistence inside the PDF itself.

The reserved embedded attachment name is:

- `pdfcraft-crop-review.json`

When present, that attachment stores UTF-8 JSON using the same top-level field names as the external project JSON:

```json
{
  "schema_version": 1,
  "source_pdf": "",
  "crops": []
}
```

The embedded-store rules are:

- `schema_version` is required and must match the same schema version used by external project persistence
- `crops` carries the same canonical ordered array of persisted crop items
- `source_pdf` is the empty string `""` for embedded persistence
- the empty `source_pdf` value is intentional because embedded persistence mirrors crop metadata only, not copied `source.pdf` bytes

On open, the engine looks up `pdfcraft-crop-review.json` in the PDF `EmbeddedFiles` name tree, decodes it as UTF-8 JSON, and restores the crop-review store when decoding and schema validation succeed.

If the reserved attachment is missing or invalid, opening falls back to an empty crop-review store.

On save or full save:

- if the crop-review store is empty, the engine removes the reserved attachment
- otherwise, the engine replaces any prior reserved attachment with a new `pdfcraft-crop-review.json`
- the written attachment description is `PdfCraft embedded crop review workspace`
- the written attachment MIME type is derived from the attachment file name

## Relationship Between External And Embedded Persistence

The external project folder and the embedded PDF attachment mirror describe the same crop-review data model, but they are not interchangeable workspace shapes.

The contract is:

- the external `source.pdf` plus `project.json` folder remains the canonical self-contained relocatable workspace
- the embedded PDF attachment is an implemented mirror of crop metadata inside the PDF
- both persistence forms use `schema_version` plus the same persisted crop-item fields
- only the external project folder carries copied `source.pdf` bytes and therefore stands alone after relocation
- the embedded attachment intentionally uses `source_pdf: ""` and must not be documented as a second relocatable package format

Stated another way: embedded persistence is a document-carried mirror of the crop dataset, while the project folder remains the canonical external workspace contract.

## Relationship To Later Export

This workspace is upstream of export.

Later export must consume the stored crop item list from the crop-review workspace rather than a transient drag gesture or ad hoc viewer selection.

The package contract remains defined by `docs/fork/snapshot-export-spec.md`.

The rectangle/export/OCR relationship remains defined by `docs/fork/rectangle-driven-export.md`.

This note does not widen the snapshot package schema. For v1:

- crop item `caption` feeds manifest `caption`
- crop item `note` feeds manifest `note`
- crop item `kind` remains `"snapshot"`
- export iterates the stored crop items in canonical order
- export reuses the existing snapshot package writer and OCR rules instead of inventing a second export format

The persistent crop-review workspace and the original snapshot tool are therefore separate product surfaces that may share lower-layer export machinery without sharing the same UI contract.

## Revision History

- 2026-10-28 — Clarified that the canonical relocatable workspace remains `source.pdf` plus `project.json`, documented the implemented embedded PDF attachment mirror, and locked the reserved attachment name plus the shared persisted JSON field shape.
- 2026-10-26 — Created the initial crop-review workspace contract for `004_crop-review-workspace`, locking the workspace boundary, right-side panel behavior, field list, geometry rule, canonical ordering, project-folder shape, and export relationship.
