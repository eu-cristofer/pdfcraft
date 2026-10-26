# 004_crop-review-workspace — Motivation

## Problem

The rectangle-driven export experiment in `003_export-flow` proved out the lower-layer package writer, automation path, and OCR-backed export seam, but it took the wrong product shape in the desktop UI. The original snapshot tool is a lightweight copy action, not a persistent review workflow. The intended user experience is closer to the existing right-side comments workflow: users need durable crop annotations, a synchronized crop list, a metadata panel, and project persistence around those annotations before export happens.

## Approach

This project introduces a dedicated crop-review workspace inside the existing PdfCraft shell instead of overloading `Edit -> Take a snapshot`. The center remains the PDF viewer with an overlay, while the right side gains two dedicated panels: `Crops` and `Metadata`. Crop rectangles become persistent per-document items with stable identifiers, canonical ordering, stored metadata, and OCR-related fields. Export reuses the engine package writer and OCR integration already built, but runs from the stored crop dataset rather than from a one-off drag gesture.

## Key decisions

- 2026-10-26 — The original snapshot tool must remain a copy-to-clipboard action; persistent crop review is a separate workflow.
- 2026-10-26 — The intended UX is a document workspace with right-side crop and metadata panels, not a standalone screen and not a transient export dialog.
- 2026-10-26 — Crop geometry must be stored in document/PDF coordinate space, never in viewport pixels.
- 2026-10-26 — Crop ordering is canonical and stable: `(page_number, rect_y, rect_x)` ascending.
- 2026-10-26 — Export remains snapshot-only in v1, but now operates from the stored crop dataset.
- 2026-10-26 — Project persistence is part of the core feature, not optional polish.

## Risks

- Overloading existing snapshot or comment code paths would blur product boundaries and regress the original tools.
- Putting the crop model only in UI-local state would make persistence and automation parity harder later.
- Selection feedback loops between overlay, list panel, and metadata panel can cause jitter or stale state unless one path owns synchronization.
- A poorly scoped persistence format could become another transient save file rather than the intended self-contained project workspace.
- OCR model availability must fail consistently across desktop and automation export surfaces.
