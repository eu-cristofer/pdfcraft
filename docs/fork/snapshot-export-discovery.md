# Snapshot Export Discovery

Last reviewed: 2026-10-25

This note records the current code paths and behavior relevant to a future snapshot export package.
It is the source of truth for Sprint 1 of `001_snapshot-export-contract`.

## What exists today

- There is no existing snapshot export package, manifest writer, or LaTeX generator in this repo.
- The closest existing pieces are split across three separate paths:
  - UI snapshot capture for clipboard raster crops in `crates/ui-egui/src/zoom_snap.rs`.
  - Engine export plumbing for page image/text export in `crates/engine/src/export.rs`.
  - Comment-summary reporting as generated plain text turned into a PDF in `crates/engine/src/lib.rs` and `crates/create/src/lib.rs`.
- Because those paths are separate today, Sprint 2+ should not assume there is already one report-item model to plug into.

## Discovery answers

### 1. Coordinate system and page rotation handling

- Current user-facing automation geometry is top-left-origin on the displayed page, 1-based by page number. This is documented in `crates/automation/README.md`.
- Current page-model and edit internals use display space and user space distinctly:
  - `pdfcraft-model` defines `view_matrix_for(rotation, crop)` as the transform from display space to PDF user space.
  - Display space is points, after `/Rotate` and `/UserUnit`, with origin at the bottom-left of the displayed page in model/edit internals.
  - Some UI/automation surfaces expose top-left-origin coordinates and convert them before edits.
- `Document::object_move_offset` in `crates/engine/src/lib.rs` is a concrete example of converting displayed top-left-origin motion into user-space offsets while accounting for page rotation.
- The current UI snapshot tool does not store a semantic rectangle model. `PdfCraftApp::snapshot` in `crates/ui-egui/src/zoom_snap.rs` takes a `view_rect` in page-view points, converts it to a renderer tile, and rasterizes that tile directly from the page.
- For exported snapshot evidence, the safest v1 contract is therefore:
  - store rectangles in one explicit coordinate space only;
  - state whether they are top-left-origin or bottom-left-origin;
  - state whether they are before or after page rotation;
  - make the renderer conversion explicit and shared.
- Recommendation for later sprints: adopt the automation-facing convention for the contract boundary, because it is already the repo's external geometry language, then normalize internally through one shared conversion path.

Relevant code:

- `crates/automation/README.md`
- `crates/model/README.md`
- `crates/model/src/lib.rs`
- `crates/engine/src/lib.rs`
- `crates/ui-egui/src/zoom_snap.rs`

### 2. Current snapshot ordering source and desired export ordering

- There is no current snapshot-export item list.
- The nearest existing ordered reporting source is the comment-summary path.
- Annotation discovery order today is established in `crates/render/src/inspect.rs`:
  - annotations are collected into `DocInfo.annotations`;
  - then sorted by `(page ascending, rect[3] descending)`;
  - then reply links are resolved.
- `Annotation.rect` is stored in PDF user space, and `Annotation.index` preserves the position inside the page's `/Annots` array.
- `comment_summary` in `crates/engine/src/lib.rs` builds its numbered list from `DocInfo.annotations` in that inspected order. Numbering resets per page before any secondary sort. Secondary sorts by author/date/type reorder the numbered top-level comments after numbering.
- The current UI snapshot tool has no persisted ordering at all. It is an immediate action over the currently dragged area.
- For future snapshot export, ordering should not be derived implicitly from render order, annotation array order, filename sorting, or caption text.
- Recommendation for later sprints: build one normalized ordered item list first, then derive all IDs, filenames, manifest entries, and LaTeX/report blocks from that list only.

Relevant code:

- `crates/render/src/inspect.rs`
- `crates/render/src/lib.rs`
- `crates/engine/src/lib.rs`
- `crates/engine/src/tests.rs`

### 3. Whether report comments/captions are plain text or trusted LaTeX

- The current report path is plain text, not LaTeX.
- `Session::summarize_comments` builds a text report string with `comment_summary(...)`.
- `Session::create_report` then calls `pdfcraft_create::from_text_escaped(...)` to make a PDF from that text.
- `from_text_escaped` in `crates/create/src/lib.rs` is explicit about the current trust model:
  - input text is treated as plain text;
  - unsupported characters are escaped as `[U+XXXX]`;
  - the report adds a note saying that escaping happened.
- This means the current repo behavior treats generated report content as plain text that must survive arbitrary Unicode, not as trusted markup.
- There is no existing equivalent code path that treats comment contents, captions, or report text as trusted LaTeX.
- Decision for the future export contract: unless a new explicit field is introduced for trusted markup, existing comment contents/captions should be treated as plain text data and any LaTeX output should be a derived rendering layer with its own escaping rules.

Relevant code:

- `crates/engine/src/lib.rs`
- `crates/create/src/lib.rs`
- `crates/create/src/tests.rs`

### 4. What `source.pdf` must represent with unsaved edits

- The repo already distinguishes between:
  - the working file: `Document.bytes`, which is what Save writes;
  - the display copy: `Document.display`, which may add generated appearances for viewing/export.
- `display_bytes(...)` in `crates/engine/src/lib.rs` adds missing comment and field appearances for viewing. Those extra appearances are for display/export convenience only and are not what Save necessarily writes.
- `Document::export_source()` currently returns `self.display.clone()`, not the working file, because image/text export should match what the user sees on screen.
- `Session::save_bytes()` returns the working file bytes that Save would write.
- `autosave_snapshots()` also persists the working file, confirmed by `crates/engine/src/tests.rs`.
- Existing comments-without-appearance coverage confirms the split: display/export sees generated appearances, while Save writes no synthetic `/AP` entries.
- For a reproducible snapshot-export package, `source.pdf` should mean the actual working file for the document state being exported, not the display copy.
- Rationale:
  - it is the true document state with unsaved edits included;
  - it matches `save_bytes()` and autosave semantics;
  - it avoids serializing display-only generated artifacts as though they were canonical document content.
- The package may still render snapshot images from display-equivalent bytes if the contract says so, but that must be spelled out separately from `source.pdf`.

Relevant code:

- `crates/engine/README.md`
- `crates/engine/src/lib.rs`
- `crates/engine/src/export.rs`
- `crates/engine/src/tests.rs`

### 5. Current crop/image format defaults and whether to preserve or change them

- Current UI snapshot capture is raster-only and clipboard-oriented:
  - it renders a selected tile with the current zoom and pixels-per-point;
  - it reads from `doc.bytes`, not `doc.display`;
  - it stores raw RGBA pixels in memory as `last_snapshot`.
- Current page export is page-wide, not rectangular crop export:
  - `Exporter::png()` renders a full page to PNG;
  - `Exporter::image()` supports PNG, JPEG, or TIFF;
  - PNG uses adaptive filtering and deflate level 3;
  - JPEG/TIFF encode explicit dpi metadata;
  - PNG intentionally does not promise exact physical-size metadata.
- Current image/text export consumes `Document.display` through `export_source()`, so it matches on-screen appearances.
- There is no existing persisted rectangular snapshot-image export path to preserve verbatim.
- Because the current UI snapshot tool is clipboard-focused and tied to live zoom, its raster behavior should not be treated as a stable export contract.
- Recommendation for later sprints:
  - define one deterministic render scale/format for exported snapshot crops, independent of transient UI zoom;
  - prefer PNG for v1 cropped evidence unless a stronger package-size reason appears, because it is lossless and already supported by the export stack;
  - make display-vs-working-file render semantics explicit in the contract.

Relevant code:

- `crates/ui-egui/src/zoom_snap.rs`
- `crates/engine/src/export.rs`
- `crates/engine/src/lib.rs`

## Summary decisions for Sprint 2

- No hidden implementation already solves snapshot export; later sprints will define new package-specific structures.
- External geometry for the export contract should follow one explicit, documented convention and normalize through a shared transform path.
- Export ordering must come from one normalized ordered item list, not from incidental renderer/UI/doc-array order.
- Report text today is plain text data. Any LaTeX should be derived output with explicit escaping/trust rules.
- `source.pdf` should represent the working file for the exported state, including unsaved edits, not the display copy.
- Exported snapshot rasters should use deterministic rendering settings, not live UI zoom behavior.
