# Sprint 5 — Release (gated on the suite name, U1)

## Goal

Fill in the final names and publish the first public pre-release of `<Suite>` PDF Review ("v0.1"), using the fork's tag scheme `0.14.0-unsigned.<YYYYMMDD>`. Smoke-test it on Windows (non-admin) and macOS.

## Gate

**STOP with `BLOCKED: waiting for U1 (suite name)`** if the owner has not supplied the suite and app name and icon. Check `eu-cristofer/inspection-suite/docs/journey.md`, "User inputs", U1.

## Files

- **`crates/ui-egui/src/branding.rs`:** final `SUITE_NAME`, `APP_NAME` and `APP_ID` (`com.costainspection.<suitename>-pdf`, lowercase, no spaces).
- **The packaging files changed in sprint 2** (each marked `keep in sync with branding.rs`): the same values.
- **The `ai.storyteller` occurrences** listed in the CHANGELOG by sprint 2: resolve the remaining ones, or document why each must stay (e.g. upstream-internal ids not visible to users).
- **Final icon assets**, if the owner supplies them: replace the placeholders and update `ATTRIBUTION.toml`.
- **`CHANGELOG.md`:** a dated section "`<date>` (on top of upstream 0.14.0) — `<Suite>` PDF Review v0.1", summarising sprints 1–5.
- **Optional repo rename:** `gh repo rename <suite>-pdf -R eu-cristofer/pdfcraft`. Only if the owner asks; GitHub redirects the old URL. Update `HOMEPAGE_URL` to match.

## Steps

1. Run the acceptance commands of sprints 1–2 again, with the final names.
2. Push the branch and open a PR into the fork's `main`. Merge when CI is green.
3. Dispatch `unsigned-release.yml` on `main` with tag `0.14.0-unsigned.<YYYYMMDD>`.
4. **Smoke test** on Windows (a non-admin account, from the portable zip) and on macOS:
   - launch, and check the window title shows `APP_NAME` and About shows `BASED_ON`;
   - open a sample PDF (synthetic, no client data);
   - create 3 crops in the crop-review workspace and edit their metadata;
   - save the project folder, close, reopen;
   - export a package with `report.docx` and OCR enabled;
   - check that `manifest.json` lists `item_1..item_3` in order, that `evidence/` has 3 PNGs, and that `ocr_text` is filled in.
5. Record the checklist results in the suite's `docs/journey.md` (G1 entry) and in the CHANGELOG.

## Reuse

- `docs/techind/update-fork-and-release.md` (moved to `docs/fork/techind/` in sprint 4): the fork's release runbook and tag methodology.

## Tests

The smoke-test checklist above (manual, recorded).

## Acceptance

```bash
cargo xtask assets
timeout 1200 cargo test -p pdfcraft-ui-egui -j 2
! grep -rn "<Suite>\|<suite>" crates apps packaging
! grep -rni "getartcraft\|docs/brand" crates apps packaging
gh release view 0.14.0-unsigned.<YYYYMMDD> -R eu-cristofer/pdfcraft
```

## Definition of done

- The pre-release is published, with the macOS dmg and the Windows portable zip.
- The smoke checklist passed and is recorded.
- G1 is marked in the suite journey.
- The tracker is ticked, and the project is marked `Status: ready to archive`.

## Assumes

- Sprints 1–4 are on this branch.
- U1 is supplied.
- `gh` is authenticated as `eu-cristofer`.
