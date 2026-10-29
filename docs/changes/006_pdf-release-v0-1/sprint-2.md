# Sprint 2 — App icon, ids and packaging metadata

## Goal

Remove the remaining ArtCraft and Storyteller identity from the app icon, bundle ids, packaging metadata and repository docs, and delete `docs/brand/`.

## Files

- **App icon:** `apps/pdfcraft/src/main.rs:54-56` (`APP_ICON_PNG` via `include_bytes!` of `assets/app-icon/pdfcraft-1024.png` and `assets/app-icon/hicolor/256x256/apps/ai.storyteller.pdfcraft.png`).
  - Replace the icon images with contributor-original placeholders derived from `assets/brand-suite/suite-mark.svg` (sprint 1), generated with `packaging/icons.sh` if it supports that; otherwise add the PNGs directly.
  - Rename the hicolor file to `<APP_ID>.png` only if the packaging scripts derive it from a single variable; otherwise keep the file names and only replace the content (smallest change).
  - Update the `ATTRIBUTION.toml` entries for every replaced asset.
  - The Windows document icon (`assets/app-icon/pdfcraft-document.*`) is Lucide plus contributor art and does not use the trademark. Leave it.
- **Packaging metadata** (replace `getartcraft.com` URLs with `branding::HOMEPAGE_URL`'s value, and change the publisher/vendor to "Costa Inspection"):
  - `packaging/windows/pdfcraft.wxs:41-42` (`ARPURLINFOABOUT`, `ARPURLUPDATEINFO`) and the Manufacturer attribute;
  - `packaging/linux/nfpm.yaml` (`homepage`, `vendor`, `maintainer`);
  - `packaging/linux/ai.storyteller.pdfcraft.metainfo.xml.in` (`url type="homepage"`, `developer`);
  - `packaging/macos/Credits.rtf` (the getartcraft hyperlink, which becomes the "Based on PdfCraft" text);
  - macOS bundle identifier and name: find with `grep -rn "ai.storyteller" packaging apps`. Change them to the APP_ID placeholder **only** where a single source exists; list the remaining occurrences in the CHANGELOG for sprint 5.
- **Delete `docs/brand/`** and its `ATTRIBUTION.toml` entries (`kind = "logo"`, `LicenseRef-Storyteller-Trademark`).
- **`NOTICE`:**
  - remove the ArtCraft brand paragraph;
  - add at the top: "`<Suite>` PDF Review is a modified version of PdfCraft (https://github.com/storytold/pdfcraft), Copyright (c) 2026 ArtCraft Team and the PdfCraft contributors, MIT OR Apache-2.0. Modifications Copyright (c) 2026 Costa Inspection.";
  - keep every third-party section unchanged.
- **`CHANGELOG.md`:** a bullet noting the new app id, which means a fresh settings and recovery directory.

## Reuse

- `packaging/icons.sh`, `cargo xtask assets`, and the existing `ATTRIBUTION.toml` format.
- `branding.rs` constants from sprint 1. Packaging files are not Rust, so copy the values and add a comment `# keep in sync with crates/ui-egui/src/branding.rs`.

## Tests

- `cargo xtask assets` passes. It verifies that each asset has an attribution entry.
- The packaging lint logic, if runnable locally: see `.github/workflows/packaging-lint.yml` for the commands.

## Acceptance

```bash
cargo xtask assets
timeout 1200 cargo test -p pdfcraft -j 2
timeout 1200 cargo clippy -p pdfcraft -j 2 --all-targets -- -D warnings
! grep -rni "getartcraft\|artcraft-logo\|artcraft-mark\|LicenseRef-Storyteller" crates apps packaging assets NOTICE ATTRIBUTION.toml
test ! -d docs/brand
```

## Definition of done

- All acceptance commands pass.
- `NOTICE` states the fork origin.
- Any remaining `ai.storyteller` ids are listed in the CHANGELOG for sprint 5.
- The tracker is ticked.

## Assumes

- Sprint 1: `branding.rs` and `assets/brand-suite/*.svg` exist; no code references `docs/brand/`.

## Addendum (2026-10-29, found while running sprint 1)

The ArtCraft community links are code in the engine crate, so they can't use `crates/ui-egui/src/branding.rs` (layering: engine must not depend on ui-egui). They also suggest that ArtCraft supports this fork, which the brand licence forbids. This sprint therefore also covers:
- `crates/engine/src/links.rs`: `DISCORD`, `WEBSITE`, `APP_PAGE`, `GITHUB` and the `LINKS` labels ("Join the ArtCraft Discord", "ArtCraft website", "PdfCraft web page"):
  - point them at the fork's own resources (`https://github.com/eu-cristofer/pdfcraft`, plus its Issues page for help);
  - or drop entries, keeping the registry commands consistent (`help.*` in the command registry).
  - Keep one plain "PdfCraft on GitHub" (upstream) link as attribution.
- The home screen card "Join the ArtCraft community" / "Get help, share feedback and follow development on Discord." (`crates/ui-egui/src/home.rs`) and the top-bar Discord button.
- `crates/ui-egui/tests/it/links.rs`: update the expectations to the new links.
