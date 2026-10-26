# Snapshot Export Spec

Last reviewed: 2026-10-25

## Status

This document defines the PdfCraft snapshot export package contract for v1.

`manifest.json` is the canonical contract.
All other package files, including `report.tex`, are derived artifacts.

## Scope

This spec covers a self-contained export package for snapshot evidence.

v1 is snapshot-only, but each exported item still carries `kind = "snapshot"` so the format can grow to additional evidence kinds later without rewriting the package model.

This spec does not define how the UI gathers snapshots or how an upstream caller chooses the item order.
It defines the package that an exporter must write once it has received one normalized ordered item list.

## Normative language

The key words "MUST", "MUST NOT", "REQUIRED", "SHOULD", and "MAY" are to be interpreted as normative requirements for this contract.

## Package contents

A valid v1 package MUST contain exactly these required artifacts:

- `manifest.json`
- `source.pdf`
- `report.tex`
- `evidence/`
- one PNG file per manifest item at `evidence/<id>.png`

A package MAY contain additional non-canonical convenience files in later versions, but v1 exporters SHOULD keep the package minimal and MUST NOT require any file other than the ones listed above.

The package root layout for v1 is:

```text
<export-root>/
  manifest.json
  source.pdf
  report.tex
  evidence/
    item_1.png
    item_2.png
    ...
```

## Canonical artifact rules

- `manifest.json` is the only canonical description of exported evidence.
- `report.tex` MUST be generated from `manifest.json` and MUST NOT introduce its own ordering, IDs, or evidence data not present in the manifest.
- `source.pdf` MUST represent the working file bytes for the exported document state, including unsaved edits that are part of that state.
- Snapshot PNGs are derived renderings and MAY reflect display-only appearance generation needed to match on-screen rendering.
- If any derived artifact disagrees with `manifest.json`, the manifest is authoritative and the package is invalid.

## Versioning

The top-level manifest field `manifest_version` MUST be a string.

v1 uses:

```json
"manifest_version": "1.0"
```

Versioning rules:

- A change that preserves all v1 required semantics and only adds optional fields is a minor version change.
- A change that changes required semantics, removes fields, changes coordinate meaning, or adds new required behavior is a major version change.
- v1 consumers MUST reject manifests whose major version is not `1`.
- v1 consumers MAY reject a higher minor version if they cannot safely ignore newly added fields.
- v1 exporters MUST emit `1.0`.

## Deterministic package rules

An exporter MUST derive all package artifacts from one shared ordered item list.

The exporter MUST NOT re-sort that list separately for:

- manifest entries
- item IDs
- image filenames
- LaTeX figure order
- LaTeX labels
- cross-references

The ordered `items` array in `manifest.json` is the canonical order for the package.

No consumer may infer order from:

- filesystem enumeration
- lexical filename sorting
- page number sorting
- caption text
- render order
- annotation-array order

## Deterministic item IDs

IDs are positional and package-local.

For an item at 1-based position `N` in the canonical `items` array, the ID MUST be:

```text
item_N
```

Examples:

- first item: `item_1`
- second item: `item_2`
- tenth item: `item_10`

ID rules:

- IDs MUST start at `item_1`.
- IDs MUST be contiguous with no gaps.
- IDs MUST be unique within the package.
- IDs MUST be derived only from the canonical ordered item list.
- IDs MUST NOT be derived from captions, page numbers, annotation IDs, or filenames.
- The image path for each item MUST be `evidence/<id>.png`.

## Manifest schema

`manifest.json` MUST be valid UTF-8 JSON encoded as an object.

For deterministic output, exporters SHOULD emit:

- two-space indentation
- `\n` line endings
- a trailing newline
- object members in the order shown below

Consumers MUST treat JSON object member order as non-semantic.

### Top-level fields

A v1 manifest MUST contain these top-level fields:

```json
{
  "package_kind": "pdfcraft.snapshot_export",
  "manifest_version": "1.0",
  "source_pdf": {
    "path": "source.pdf",
    "state": "working-copy"
  },
  "coordinate_space": {
    "page_origin": "top-left",
    "page_space": "display",
    "units": "pt",
    "page_rotation": "already-applied"
  },
  "rendering": {
    "snapshot_source": "display-equivalent",
    "image_format": "png",
    "pixels_per_point": 2.0
  },
  "items": []
}
```

Field rules:

- `package_kind`: REQUIRED. MUST be the exact string `pdfcraft.snapshot_export`.
- `manifest_version`: REQUIRED. MUST be the exact string `1.0`.
- `source_pdf`: REQUIRED. Describes the semantic meaning of `source.pdf`.
- `coordinate_space`: REQUIRED. Declares the fixed coordinate contract for all item rectangles in the package.
- `rendering`: REQUIRED. Declares the fixed rendering contract for all snapshot images in the package.
- `items`: REQUIRED. Array of exported evidence items in canonical order.

### `source_pdf`

`source_pdf` fields:

- `path`: REQUIRED. MUST be `source.pdf`.
- `state`: REQUIRED. MUST be `working-copy`.

`state = "working-copy"` means:

- the file is the document state being exported
- unsaved edits included in that state MUST be present
- display-only generated appearances MUST NOT be serialized into `source.pdf` unless they are already part of the working file itself

### `coordinate_space`

`coordinate_space` fields are fixed for v1 and MUST be:

- `page_origin = "top-left"`
- `page_space = "display"`
- `units = "pt"`
- `page_rotation = "already-applied"`

Their meaning is:

- `top-left`: `x` and `y` are measured from the top-left corner of the displayed page
- `display`: coordinates refer to displayed page space, not raw PDF user space
- `pt`: values are in points, where 1 point = 1/72 inch
- `already-applied`: page rotation has already been applied before measuring the rectangle

### `rendering`

`rendering` fields for v1 MUST be:

- `snapshot_source = "display-equivalent"`
- `image_format = "png"`
- `pixels_per_point = 2.0`

Their meaning is:

- `display-equivalent`: snapshot rendering MUST match display semantics for the exported state, including appearance generation needed for visual fidelity
- `png`: all snapshot images are lossless PNG files
- `2.0 pixels_per_point`: exported images are rendered at a deterministic scale equal to 144 dpi

## Item schema

Each entry in `items` MUST be an object with these fields in v1:

```json
{
  "id": "item_1",
  "kind": "snapshot",
  "page": 1,
  "rect": {
    "x": 72.0,
    "y": 96.0,
    "width": 216.0,
    "height": 120.0
  },
  "image": {
    "path": "evidence/item_1.png",
    "width_px": 432,
    "height_px": 240
  },
  "caption": "Short plain-text caption",
  "note": "Optional plain-text note."
}
```

Field rules:

- `id`: REQUIRED. Must follow the deterministic ID rule above.
- `kind`: REQUIRED. MUST be the exact string `snapshot` in v1.
- `page`: REQUIRED. 1-based page number in the exported document state.
- `rect`: REQUIRED. Rectangle in the package coordinate space.
- `image`: REQUIRED. Points to the rendered evidence PNG for this item.
- `caption`: REQUIRED. Plain-text caption.
- `note`: OPTIONAL. Plain-text explanatory note. If present, it MUST be a string.

### `rect`

`rect` fields:

- `x`: REQUIRED number, `>= 0`
- `y`: REQUIRED number, `>= 0`
- `width`: REQUIRED number, `> 0`
- `height`: REQUIRED number, `> 0`

`rect` semantics:

- The rectangle is measured in displayed-page points.
- `x` increases to the right.
- `y` increases downward from the top edge of the displayed page.
- The rectangle refers to the rotated displayed page, not the unrotated PDF user-space page.
- The rectangle MUST NOT depend on transient UI zoom.
- Fractional point values are allowed.

### `image`

`image` fields:

- `path`: REQUIRED. MUST equal `evidence/<id>.png`.
- `width_px`: REQUIRED integer, `> 0`
- `height_px`: REQUIRED integer, `> 0`

Image semantics:

- The image file MUST exist at the declared path.
- The file MUST be a PNG.
- The manifest dimensions MUST match the actual raster dimensions of the file.
- The image MUST be rendered from the item's `rect` using the package rendering settings.
- Exporters SHOULD compute `width_px` and `height_px` by multiplying point dimensions by `pixels_per_point` and rounding to the nearest integer, with halves rounded up.
- Consumers MUST trust the actual PNG dimensions and validate that they match the manifest.

## Coordinate contract

The package boundary uses one coordinate language only.

v1 rectangle coordinates are:

- top-left origin
- 1-based page numbering
- displayed-page space
- post-rotation
- points

This means a caller can read `rect` without knowing PDF user-space conventions.

Implementation guidance required by this spec:

- Export code MUST normalize into one shared conversion path from this external contract to renderer or PDF internals.
- Export code MUST NOT duplicate ad hoc coordinate math across separate call sites.
- A rotated page MUST still export the rectangle the user selected on the displayed page.
- A consumer that wants raw PDF user-space coordinates must convert from the exported contract; the manifest does not store raw PDF user-space rectangles.

## Source PDF semantics

`source.pdf` is not a display cache.

`source.pdf` MUST be the working file bytes for the exported state.

Consequences:

- unsaved edits that are part of the exported state MUST be present in `source.pdf`
- the file MUST match the document state that Save would write for that moment
- display-only generated appearances used for rendering MUST NOT redefine what `source.pdf` means

Snapshot image rendering is separate from `source.pdf` semantics.

For visual fidelity, snapshot PNGs MAY be rendered from a display-equivalent view of the same exported state.
If that rendering uses generated appearances or similar display-time materialization, that behavior applies only to derived outputs such as PNGs and `report.tex`.
It does not change the meaning of `source.pdf`.

## LaTeX generation contract

`report.tex` is a derived artifact generated from `manifest.json`.

### Required derivation rules

- Figures MUST appear in the same order as `manifest.items`.
- Each figure MUST include the item's PNG at `image.path`.
- Each figure label MUST be derived from the item ID only.
- The v1 label format MUST be `fig:<id>`.
- Captions and notes in LaTeX MUST come from manifest plain-text fields only.
- The LaTeX generator MUST NOT invent its own IDs, labels, sort order, or filenames.

### Plain text and trusted LaTeX

In v1, manifest text fields are plain text data.

v1 defines no trusted-LaTeX fields.

Rules:

- `caption` is plain text and MUST be LaTeX-escaped before insertion into `report.tex`.
- `note` is plain text and MUST be LaTeX-escaped before insertion into `report.tex`.
- Exporters MUST NOT infer trusted markup from a string's contents.
- Characters with LaTeX meaning, such as `\`, `{`, `}`, `%`, `&`, `$`, `#`, `_`, `^`, and `~`, MUST be escaped or otherwise emitted safely as plain text.
- Plain-text fields MUST NOT be parsed as LaTeX commands.
- Future trusted markup, if added, MUST use new explicitly named fields and a versioned contract change.

### UTF-8

`report.tex` MUST be UTF-8 text.

The export contract does not require compiled PDF output.
It requires deterministic `.tex` generation.
If a chosen LaTeX pipeline cannot represent exported plain text safely, the exporter MUST fail the export rather than silently changing manifest data.

## Validation rules

A v1 package is valid only if all of the following are true:

- `manifest.json` parses as JSON.
- `package_kind` is exactly `pdfcraft.snapshot_export`.
- `manifest_version` is exactly `1.0`.
- `source_pdf.path` is exactly `source.pdf`.
- `source_pdf.state` is exactly `working-copy`.
- `coordinate_space` matches the fixed v1 contract exactly.
- `rendering` matches the fixed v1 contract exactly.
- `items` is a non-empty array.
- every item has a unique `id`
- every item `id` matches its 1-based position in the `items` array
- every item `kind` is exactly `snapshot`
- every item `page` is an integer greater than or equal to `1`
- every `rect` has numeric fields with positive width and height
- every `image.path` matches `evidence/<id>.png`
- every referenced image file exists
- every referenced image file is a PNG
- every image's actual pixel size matches `width_px` and `height_px`
- `report.tex` exists
- `report.tex` figure order, labels, and image references are derivable from the manifest without contradiction
- `source.pdf` exists

Consumers MAY perform stricter checks, but they MUST NOT reinterpret invalid data as valid by guessing missing semantics.

## Failure behavior

Export is all-or-nothing.

If an exporter cannot satisfy this spec, it MUST fail the export and MUST NOT present the package as complete.

Failure cases include:

- invalid or incomplete ordered item input
- invalid coordinates
- missing source document state
- render failure for any item
- inability to write required files
- inability to generate valid `report.tex`
- any mismatch between manifest data and derived artifact paths or IDs

An implementation SHOULD stage the package in a temporary directory and move it into place only after validation succeeds.
If staging is not possible, the implementation MUST still ensure that a partial package is not reported as a successful export.

## Future expansion stance

v1 deliberately reserves room for additional evidence kinds.

Rules for forward evolution:

- v1 exporters MUST emit only `kind = "snapshot"`.
- Future versions MAY add new `kind` values such as highlights, text extracts, or other evidence forms.
- New evidence kinds MUST NOT change the meaning of existing snapshot fields inside v1.
- Consumers MUST NOT assume that all future items will share the snapshot-specific `rect` and `image` shape.
- Future trusted-markup support MUST use explicit new fields rather than changing the meaning of existing plain-text fields.
- Future non-LaTeX outputs MAY be added, but they remain derived artifacts unless a later version explicitly promotes them to canonical status.

## Worked examples

### Example package tree

```text
case-export/
  manifest.json
  source.pdf
  report.tex
  evidence/
    item_1.png
    item_2.png
    item_3.png
```

### Example `manifest.json`

```json
{
  "package_kind": "pdfcraft.snapshot_export",
  "manifest_version": "1.0",
  "source_pdf": {
    "path": "source.pdf",
    "state": "working-copy"
  },
  "coordinate_space": {
    "page_origin": "top-left",
    "page_space": "display",
    "units": "pt",
    "page_rotation": "already-applied"
  },
  "rendering": {
    "snapshot_source": "display-equivalent",
    "image_format": "png",
    "pixels_per_point": 2.0
  },
  "items": [
    {
      "id": "item_1",
      "kind": "snapshot",
      "page": 2,
      "rect": {
        "x": 72.0,
        "y": 128.0,
        "width": 216.0,
        "height": 96.0
      },
      "image": {
        "path": "evidence/item_1.png",
        "width_px": 432,
        "height_px": 192
      },
      "caption": "Signature block on page 2",
      "note": "Signer name is present, but the signature field is still empty."
    },
    {
      "id": "item_2",
      "kind": "snapshot",
      "page": 4,
      "rect": {
        "x": 90.5,
        "y": 244.0,
        "width": 180.0,
        "height": 110.0
      },
      "image": {
        "path": "evidence/item_2.png",
        "width_px": 360,
        "height_px": 220
      },
      "caption": "Comment callout near revised paragraph",
      "note": "Rendered from display-equivalent state so the comment appearance matches what the user saw at export time."
    },
    {
      "id": "item_3",
      "kind": "snapshot",
      "page": 7,
      "rect": {
        "x": 64.0,
        "y": 88.0,
        "width": 256.0,
        "height": 144.0
      },
      "image": {
        "path": "evidence/item_3.png",
        "width_px": 512,
        "height_px": 288
      },
      "caption": "Highlighted total amount in payment summary",
      "note": "Coordinates use top-left origin on the displayed page after rotation has already been applied."
    }
  ]
}
```

### Example `report.tex` snippet

```tex
\section*{Snapshot Evidence}

\begin{figure}[htbp]
\centering
\includegraphics[width=\linewidth]{evidence/item_1.png}
\caption{Signature block on page 2}
\label{fig:item_1}
\end{figure}

\paragraph{Note.}
Signer name is present, but the signature field is still empty.

\begin{figure}[htbp]
\centering
\includegraphics[width=\linewidth]{evidence/item_2.png}
\caption{Comment callout near revised paragraph}
\label{fig:item_2}
\end{figure}

\paragraph{Note.}
Rendered from display-equivalent state so the comment appearance matches what the user saw at export time.
```

## Summary

The v1 snapshot export package is intentionally strict:

- one canonical manifest
- one canonical ordered item list
- deterministic package-local IDs
- explicit coordinates
- explicit source-PDF semantics
- deterministic PNG rendering settings
- plain-text manifest fields with escaped LaTeX derivation
- fail-closed validation behavior

That strictness is the mechanism that keeps the format reproducible now and expandable later.
