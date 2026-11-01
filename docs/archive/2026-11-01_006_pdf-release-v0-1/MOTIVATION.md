# 006_pdf-release-v0-1 — Motivation

## Problem

The crop-review and evidence-export work (projects 001–005) is complete and in daily use. Two problems block a public release from this fork:

1. **Trademarks.**
   - The app embeds the ArtCraft wordmark and mark (`crates/ui-egui/src/widgets.rs:559-575`, `include_bytes!` of `docs/brand/*.svg`).
   - `docs/brand/LICENSE-brand.txt` forbids them in forks and modified versions, and Apache-2.0 §6 grants no trademark rights.
   - The fork's `unsigned-release.yml` already publishes builds.
2. **Broken Windows OCR.** The release workflow zips only `pdfcraft.exe` and `pdfcraft-cli.exe`, without the `models\` folder, so Scan & OCR fails in the released zip. `packaging/windows/package.ps1` already builds a correct portable zip.

## Approach

- **Branding in one place.** A single `branding` module drives all names, ids and URLs, so that filling in the suite name (U1) is a one-file change.
- **Replace, don't hide.** The ArtCraft assets are deleted and replaced with contributor-original assets that have `ATTRIBUTION.toml` entries.
- **Reuse the existing packaging** instead of writing a new zip step.
- **Shrink the footprint on upstream files.** Fork docs move to `docs/fork/`, so the later upstream port (P5) touches fewer upstream-owned files.
- **Release with the fork's existing tag scheme** `<upstream-version>-unsigned.<YYYYMMDD>` (see `CHANGELOG.md`). The product is labelled "v0.1" in the release title.

## Key decisions

| Date | Decision | Ref (suite `docs/decisions.md`) |
|---|---|---|
| 2026-10-28 | The fork is a downstream product | D-001 |
| 2026-10-28 | MIT OR Apache-2.0, in-house use free of charge | D-002 |
| 2026-10-28 | Remove ArtCraft marks; own suite name | D-003 |
| 2026-10-29 | Ship first; the upstream sync comes after release (P5) | D-005 |
| 2026-10-29 | All repos on `eu-cristofer`; app id prefix `com.costainspection` | D-012 |

## Risks

| Risk | Mitigation |
|---|---|
| A brand reference is missed (packaging, docs, About box) | Grep gate in sprints 1–2 and 5: `artcraft` / `getartcraft` / `docs/brand` return nothing outside the "based on PdfCraft" notice |
| A new app id moves the settings/recovery directory | Expected for a renamed product; noted in `CHANGELOG.md` |
| `package.ps1` expects signing or WiX in CI | Run it in portable-only mode with signing skipped; if not supported, add a flag in the smallest possible change |
| Panel snapshot tests change with the new mark | Update the snapshots deliberately and review the images |
| The suite name (U1) is not chosen by sprint 5 | Sprint 5 is gated; sprints 1–4 work with `<Suite>` placeholders |
