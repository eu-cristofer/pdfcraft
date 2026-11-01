# Sprint 1 — Branding module and removal of the in-app marks

## Goal

Remove every in-app use of the ArtCraft trademarks and route the app's name, id and URLs through one `branding` module. The values may still be placeholders.

## Files

- **Create `crates/ui-egui/src/branding.rs`.** Public constants with doc comments:
  - `SUITE_NAME = "<Suite>"`, `APP_NAME = "<Suite> PDF Review"`;
  - `APP_ID = "com.costainspection.<suite>-pdf"` (placeholder, lowercase);
  - `HOMEPAGE_URL = "https://github.com/eu-cristofer/pdfcraft"`;
  - `BASED_ON = "Based on PdfCraft by the ArtCraft Team and the PdfCraft contributors (MIT OR Apache-2.0)"`. A plain-text attribution like this is allowed, because it uses the name only to describe origin.
  - A unit test asserts `APP_NAME` starts with `SUITE_NAME`.
- **`crates/ui-egui/src/lib.rs`:** add `pub mod branding;`.
- **`crates/ui-egui/src/widgets.rs:559-575`:**
  - delete the ArtCraft wordmark and mark functions, which `include_bytes!` `docs/brand/artcraft-logo-white.svg`, `artcraft-logo.svg` and `artcraft-mark.svg`;
  - add `suite_mark(ui, size)` and `suite_wordmark(ui, height)` that load `assets/brand-suite/suite-mark.svg` and `suite-wordmark.svg` (dark/light variants if the old function had them), with `alt_text(branding::APP_NAME)`.
- **Create `assets/brand-suite/suite-mark.svg` and `suite-wordmark.svg`:** simple contributor-original placeholders (a geometric mark plus a text-free wordmark shape; no fonts embedded as outlines from any non-OFL font). Add `ATTRIBUTION.toml` entries (`kind = "logo"`, license `MIT OR Apache-2.0`, author "Costa Inspection") following the existing entry format in `ATTRIBUTION.toml`. Regenerate `ATTRIBUTION.md` if `cargo xtask assets` requires it.
- **Every call site** of the removed functions: find them with `grep -rn "artcraft_" crates apps`, and switch them to the new functions.
- **Window title and About dialog text:** find them with `grep -rn "PdfCraft" crates/ui-egui/src apps/pdfcraft/src` for user-visible strings. Use `branding::APP_NAME`, and show `branding::BASED_ON` in About.
  - Do **not** rename crates, modules or internal identifiers.
  - Do not touch the i18n catalogues beyond the strings that must change; if a string lives in a catalogue, change only the English source and leave the translations for later (note it in the CHANGELOG).

## Reuse

- The existing `egui::Image::from_bytes(uri, bytes).max_height(..).alt_text(..)` pattern in `widgets.rs:559-575`.
- The `ATTRIBUTION.toml` entry format and the `cargo xtask assets` checker (`AGENTS.md` §1).

## Tests

- In `widgets.rs` (or `branding.rs`) tests: read `widgets.rs` source via `include_str!("widgets.rs")` and assert it does not contain `docs/brand/`. Prove that this test fails by temporarily re-adding one path.
- Update the panel snapshot tests if the mark changes rendered output (`crates/ui-egui/tests/it/panel_snapshots.rs`). Review the diffs visually.

## Acceptance

```bash
timeout 1200 cargo test -p pdfcraft-ui-egui -j 2
timeout 1200 cargo clippy -p pdfcraft-ui-egui -j 2 --all-targets -- -D warnings
cargo fmt --check
cargo xtask assets
! grep -rn "docs/brand\|artcraft_" crates apps
```

## Definition of done

- All acceptance commands pass.
- No code references the ArtCraft marks.
- The CHANGELOG has a bullet under a new dated section ("on top of upstream 0.14.0").
- The tracker is ticked.

## Assumes

- Branch `feat/006-pdf-release-v0-1`, created from `feat/crop-panel-unification` @ 17649c1b, with a clean tree.
- The marks are at `crates/ui-egui/src/widgets.rs:559-575` (verify with grep; line numbers may drift).
