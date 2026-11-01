# Sprint 1b — Baseline repair (added 2026-10-29, D-015)

## Goal

Make `cargo test -p pdfcraft-ui-egui` green on the release branch by fixing the failures that **predate this project**, so v0.1 ships with a green suite. This sprint covers no new behaviour.

## Background

Measured while running sprint 1:
- On `0a8d8cd6`, without the sprint-1 changes, 24 integration tests fail.
- With sprint 1 applied, 20 fail, and none of them are new. 4 crop tests are intermittent.

| Group | Tests | Cause |
|---|---|---|
| Crop review | `crop::*` (8 stable, 4 intermittent) | egui_kittest's accessibility check: "1 widget(s) lack accessible name … ComboBox". The OCR-mode dropdown in `crates/ui-egui/src/crops_panel.rs:292` (`egui::ComboBox::from_id_salt(("crop-ocr-mode", …))`) has no label. |
| Panel snapshots | `panel_snapshots::every_panel_matches_its_snapshot` | `tests/it/panel_snapshots.rs:217` still enumerates `RightPanel::Metadata` and hits `unreachable!("metadata panel was merged into crops")`. The test was not updated when the metadata panel was merged into Crops (`08b57ba4`). |
| Fill & sign | `fill_sign::image_signature_preview_follows_pointer_at_page_size_with_zoom_and_rotation` | `tests/it/fill_sign.rs:699`: after `hover_at(pos2(5.0, 5.0))` the cursor is `PointingHand`, not `Default`. The cause has not been found yet. Something interactive is now drawn at the window's top-left, or a fork change to `fill_sign.rs` (`git log 83ba2868..17649c1b -- crates/ui-egui/src/fill_sign.rs`) changed the cursor. |
| Fonts | `editing::*` (3), `control::*` (6), `commands::*` (1) | "No glyph for … (CJK/Thai)". These need PdfCraft's optional build input `craft-fonts` (`CRAFT_FONTS_DIR`, see `CLAUDE.md`). Expected to pass with it; to be verified. |

## Files

- `crates/ui-egui/src/crops_panel.rs`: give the OCR-mode `ComboBox` an accessible name, using the label pattern other ComboBoxes in `ui-egui` use (find one with `grep -rn "ComboBox::from_label\|labelled_by" crates/ui-egui/src`). The name is `tl!("OCR mode")`, or the existing visible label text if there is one.
- `crates/ui-egui/tests/it/panel_snapshots.rs`: remove the `RightPanel::Metadata` arm. If the `RightPanel` enum still has a `Metadata` variant only for compatibility, keep it out of the snapshot list instead of panicking. If the variant is dead, say so in the commit and leave removing it to P5. Update the snapshot only if the Crops panel render changed deliberately.
- `fill_sign`: reproduce the failure alone (`cargo test -p pdfcraft-ui-egui -j 2 --test it fill_sign::image_signature_preview_follows_pointer -- --exact`).
  - Also check the same test on upstream `83ba2868`: if it fails there too, it is an upstream issue. Mark it `#[ignore = "upstream: <reason>"]` and record that in the CHANGELOG.
  - Otherwise, fix the fork change that causes the pointing hand at (5, 5).
- Fonts:
  - Clone `https://github.com/storytold/craft-fonts` to `../craft-fonts` (outside the repo, never committed), and run the 10 font tests with `CRAFT_FONTS_DIR=../craft-fonts`.
  - If they pass, document in `docs/fork/` (or in `CHANGELOG.md` if `docs/fork/` doesn't exist yet) that local full test runs need `CRAFT_FONTS_DIR`.
  - If they fail, stop and write `BLOCKED:` under this sprint's tracker line.

## Reuse

- Existing accessible-label patterns for egui widgets in `crates/ui-egui/src` (ComboBoxes in `dialogs.rs` and `forms_ui.rs`).
- The test harness helpers in `crates/ui-egui/tests/it/crop.rs`.

## Tests

- Remove no tests. Prove each code fix by reverting it and watching the related test fail again.

## Acceptance

```bash
cargo fmt --check
cargo clippy -p pdfcraft-ui-egui -j 2 --all-targets -- -D warnings
cargo test -p pdfcraft-ui-egui -j 2 --no-fail-fast                                  # 0 failed, except the 10 font tests without craft-fonts
CRAFT_FONTS_DIR=../craft-fonts cargo test -p pdfcraft-ui-egui -j 2 --test it --no-fail-fast   # 0 failed
cargo test -p pdfcraft-ui-egui -j 2 --test it crop:: -- --test-threads=1            # run 3 times: 0 failed each (intermittency gone)
```

## Definition of done

- The suite is green as above.
- The CHANGELOG has one bullet per fix.
- Any `#[ignore]` names its upstream reason.
- The tracker is ticked.

## Assumes

- Sprint 1 is committed (`494c4b95`) on `feat/006-pdf-release-v0-1`.
- The two baseline commits are `0a8d8cd6` and `9aaede9b`.
