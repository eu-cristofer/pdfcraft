# <Suite> PDF Review app icon

<img src="pdfcraft-small.svg" alt="<Suite> PDF Review app icon: a document and a magnifier on teal" width="128">

**Motif:** a document outline with a folded corner and a magnifier over its lower right: reviewing documents.
It is a placeholder until the suite's final name and icon are chosen (input U1).

**Palette:** two colours: white line work (`#ffffff`, 30 px round strokes on the 512 tile) on a full-bleed teal
field (`#0f766e`).

**Tile:** `viewBox="0 0 512 512"`, a rounded square with `rx=112` that clips everything (kept from upstream so the
packaging scripts are unchanged). Windows and Linux icons use the full-bleed tile. macOS icons put it on
Apple's grid (an 824 px body centred on a transparent 1024 px canvas).

**Provenance:** contributor-original, drawn for this fork by Costa Inspection (2026-10-29), scaled from
`assets/brand-suite/suite-mark.svg`. It replaces the PdfCraft lion icon. The file names keep upstream's
`pdfcraft*` and `ai.storyteller.pdfcraft` names so the packaging scripts need no change; the app id is renamed
in a later step. Licence: [LICENSE.txt](LICENSE.txt) (`MIT OR Apache-2.0`, like the repo).

## Files

| File | What it is |
|---|---|
| `pdfcraft.svg` | the master vector (traced at 2048 px); every PNG, `.ico` and `.icns` is rendered from it |
| `pdfcraft-small.svg` | a lighter vector (traced at 1024 px) for places where size matters, such as this README |
| `pdfcraft-1024.png` | 1024 px on Apple's grid; also the runtime Dock icon on macOS |
| `pdfcraft.icns` | macOS icon (16–1024 px) |
| `pdfcraft.ico` | Windows icon (16–256 px), embedded in `pdfcraft.exe` by `apps/pdfcraft/build.rs` |
| `pdfcraft-document.svg` | Windows PDF file icon master: a Lucide document outline, green badge and outlined PDF lettering; no font dependency |
| `pdfcraft-document.ico` | Windows PDF file icon (16–256 px), embedded as resource 2 in `pdfcraft.exe` |
| `hicolor/<n>x<n>/apps/ai.storyteller.pdfcraft.png` | Linux hicolor theme, 16–512 px; the 256 px one is the runtime icon on Windows and Linux |
| `hicolor/scalable/apps/ai.storyteller.pdfcraft.svg` | Linux scalable icon (copy of the master) |

Where it shows: `apps/pdfcraft/src/main.rs` sets the window icon (Dock, taskbar, Alt-Tab, launcher) and the
Wayland app id `ai.storyteller.pdfcraft`; `packaging/linux/ai.storyteller.pdfcraft.desktop` names the
hicolor icon.

Windows PDF files use resource 2 (`"pdfcraft.exe",-2`) when PdfCraft is chosen as the default
reader; the app keeps resource 1. The document outline is adapted from Lucide `file-text`
1.49.0 (ISC); its badge and outlined lettering are original (`MIT OR Apache-2.0`), covered by
[LICENSE-document.txt](LICENSE-document.txt). Both icons are embedded, requiring no runtime
sidecar. This provides a file type icon; first-page thumbnails still need a thumbnail provider.

## Regenerate

```sh
packaging/icons.sh        # needs resvg and python3; iconutil (macOS) for the .icns
cargo xtask assets        # then update the sha256 values in ATTRIBUTION.toml and run with --write
```
