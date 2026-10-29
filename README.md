# <Suite> PDF Review

A Rust-native PDF application for inspection work: review PDFs by cropping regions, annotating them, and exporting evidence packages.

> Based on PdfCraft by the ArtCraft team (https://github.com/storytold/pdfcraft), MIT OR Apache-2.0. This is an independent fork by Costa Inspection; it is not an ArtCraft product and is not made or endorsed by the ArtCraft Team.

Part of Costa Inspection's open-source inspection suite (https://github.com/eu-cristofer/inspection-suite), alongside an API 610 pump-performance tool.

## What this fork adds

- **Crop-review workspace**: draw crops on PDF pages, add a caption, note and OCR text per crop, and persist them in a project folder or embedded in the saved PDF.
- **Evidence-package export**: `manifest.json` (canonical), `report.tex`, optional `report.docx`, `report.marker.json` for AI pipelines, PNG crops and `source.pdf`.
- **Native-text-first OCR**: prefers the PDF's own text and falls back to bitmap OCR.
- **Windows portable build**: runs without administrator rights.

## Everything else

All other features (viewing, commenting, forms, signing, organizing pages, editing, redaction, export) come from upstream PdfCraft. See upstream's README for the full feature list and screenshots: https://github.com/storytold/pdfcraft#readme

## Download

Get builds from GitHub Releases: https://github.com/eu-cristofer/pdfcraft/releases

These are unsigned pre-releases:

- **macOS**: ad-hoc signed. The first time, right-click the app and choose Open.
- **Windows**: portable zip. Extract anywhere and run `pdfcraft.exe`. No admin rights needed; OCR models are included.

## Build from source

```sh
cargo run -p pdfcraft -- <file.pdf>
cargo build --release -p pdfcraft -p pdfcraft-cli
```

Tests are scoped per crate, as upstream does:

```sh
cargo test -p <crate> -j 2
```

Full local runs need the optional craft-fonts input:

```sh
git clone https://github.com/storytold/craft-fonts ../craft-fonts
export CRAFT_FONTS_DIR=$(cd ../craft-fonts && pwd)
```

`CRAFT_FONTS_DIR` must be an absolute path; a relative one is not resolved from the repo root.

## Documentation

- [`docs/fork/README.md`](docs/fork/README.md): fork docs (crop review, export contract, fork sync).
- `docs/`, `CLAUDE.md` and `AGENTS.md`: upstream's developer docs, unchanged from upstream.

## Help and feedback

Open an issue: https://github.com/eu-cristofer/pdfcraft/issues

## License

MIT OR Apache-2.0 (see `LICENSE-MIT` and `LICENSE-APACHE`). `NOTICE` lists third-party material.

<!-- This README is the fork's own (D-016); .gitattributes keeps it on upstream merges (merge=ours). -->
