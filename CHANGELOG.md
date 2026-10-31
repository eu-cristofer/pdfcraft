# Changelog (this fork only)

`eu-cristofer/pdfcraft` is a fork of [`storytold/pdfcraft`](https://github.com/storytold/pdfcraft).
This file tracks only the changes made **in this fork**, on top of whatever upstream version was
current at the time — it is not, and does not try to be, upstream's own changelog or release
history. Entries are keyed by date and the upstream version they were built on, since the version
number in `Cargo.toml` is upstream's, not this fork's, to assign meaning to.

See `docs/fork/techind/update-fork-and-release.md` for the sync/re-release process these entries come
from. Release tags follow that same runbook's methodology,
`<upstream-version>-unsigned.<YYYYMMDD>` (e.g. `0.14.0-unsigned.20261025`) — so a release tag and
a changelog entry always describe the same upstream lineage, just in two different places.

## 2026-10-29 (on top of upstream 0.14.0) — WitnessPoint PDF Review v0.1

The first public pre-release of this fork, as **WitnessPoint PDF Review** (Costa Inspection's WitnessPoint suite). The crop review, OCR and evidence-package export work as they did on `feat/crop-panel-unification`, with no ArtCraft marks: a branding module and the suite's own placeholder mark (sprint 1), a green test suite (1b), its own icon, links and publisher (2), a Windows portable zip with the OCR models (3), the fork's docs layout (4), and working OCR checked in every package by CI (D-017). Details are in the entries below.

- **Changed** the names to their final values (D-018): suite `WitnessPoint`, app `WitnessPoint PDF Review`, app id and macOS bundle id `com.costainspection.witnesspoint-pdf`. Upstream's bundle id would have made macOS treat this app and an installed upstream PdfCraft as the same app. The macOS app is now `WitnessPoint PDF Review.app` on a DMG volume of the same name; the window title and the Windows version info carry the new name; the MSI has its own UpgradeCode (also in its context-menu release detection), so it can never replace an installed upstream PdfCraft. The macOS artifacts now carry the release tag in their version. The release title is "WitnessPoint PDF Review — unsigned build" (`006_pdf-release-v0-1` sprint 5).
- **Fixed** two export tests that only passed while OCR could not run: the Export dialog test now expects no OCR text from its blank pages (it fails with the mismatched models of D-017), and the macOS file-association test re-encodes its captured LaunchServices binding with the new bundle path and id (`006_pdf-release-v0-1` sprint 5).
- **Fixed** crop OCR reading the wrong part of the page: the OCR tile was scaled twice, so a crop's `ocr_text` came from the area at twice its coordinates (the evidence image was right, its text was not). It affected the Crops panel, the Export dialog and `doc_export_snapshot_package`/`doc_export_crop_review_package`, and stayed hidden while OCR never worked (D-017). Found by the v0.1 smoke test; the release run was cancelled before publishing. A new engine test checks that each half of a scanned page OCRs to the same words as the original's native text; the release OCR smoke test now also exports a 3-crop package and checks each crop's text (`006_pdf-release-v0-1` sprint 5).
- **Removed** upstream's update check. At start, and from Help ▸ Check for updates, the app asked upstream PdfCraft's release feed and offered "Update and restart", which would install upstream's app over this one (found in the v0.1 smoke test of a local build). The app no longer checks at start, and Help ▸ Check for updates opens this fork's releases page in the browser (`006_pdf-release-v0-1` sprint 5).
- **Known, left on purpose:**
  - the Linux, FreeBSD and Flatpak packaging and the hicolor icon file names still use `ai.storyteller.pdfcraft` (only `StartupWMClass` moved, to match the app id). None of them ships in v0.1; the upstream port (P5) renames them;
  - the settings and recovery folder is still named `PdfCraft`, so it is shared with an upstream PdfCraft installed on the same machine. Moving it needs a migration;
  - the executables are still `pdfcraft`/`pdfcraft-cli`, and the macOS executable inside the bundle is still `PdfCraft`;
  - the MSI's shortcut and menu labels still say PdfCraft (the MSI is not published in v0.1);
  - in-app translated strings that name PdfCraft are unchanged (the About box says "Based on PdfCraft");
  - the DMG has no custom background: the committed Finder layout is bound to the old names. It opens as a plain window;
  - the icon is still the placeholder.

## 2026-10-29 (on top of upstream 0.14.0) — fixes and the work behind v0.1

- **Fixed** OCR, which read every page as junk (`1`, `T`) in all builds: commit `17649c1b` had swapped the OCR models, during the upstream `.rten` URL brownout (ocrs #282), for other Hugging Face checkpoints that don't match the recogniser. Backported the OCR part of upstream `da8cea2b`: the author-published ONNX models (`text-detection.onnx`, `text-recognition.onnx`, SHA-256 verified), the `onnx_format` reader, model lookup that prefers a complete ONNX pair and falls back to a complete `.rten` pair, and the MSI check for the new file names. The macOS CLI zip now ships the models too (it had none). The release workflow OCR-tests every package before it can publish (`packaging/ocr-smoke.sh`: the Windows portable zip, the macOS CLI zip and the `.app`), and the crop-review package test now checks the crop's native text, which it never actually exercised before (`006_pdf-release-v0-1`, D-017).

- **Fixed** the unsigned release workflow: the macOS job now runs on the standard `macos-15` runner (`macos-14` no longer acquired a hosted runner, so run 40854286030 published nothing), and a new `publish` input (off by default) makes a dispatch build-only unless it is ticked, so a test run from a feature branch cannot publish a release.

- **Moved** the fork-only docs to `docs/fork/` (with an index in `docs/fork/README.md`), and restored upstream's `AGENTS.md`, `CLAUDE.md` and `ROADMAP.md`. Upstream's `AGENTS.md` (the binding asset and provenance rules) had been replaced on 2026-10-25 by a generated agent guide, which is kept as `docs/fork/agent-notes.md`. `README.md` is now the fork's own short README (D-016); `.gitattributes` keeps it on upstream merges (`README.md merge=ours`; run `git config merge.ours.driver true` once per clone). Removed `docs/images/pdfcraft-tools.png`, which showed the ArtCraft mark. Archived sprint projects 002 (superseded), 003 and 005 (`006_pdf-release-v0-1` sprint 4).

- **Changed** the Windows build of the unsigned release workflow to the supported packaging path (`packaging/windows/package.ps1`): a portable zip with the OCR models in `models\`, `portable.txt`, licences and font notices, built with the same craft-fonts revision as official releases. No admin rights needed. Only the zip is published; the MSI waits for its own UpgradeCode in sprint 5 (`006_pdf-release-v0-1` sprint 3).

- **Replaced** the ArtCraft community links (Discord, ArtCraft website, getartcraft.com app page) with this build's own: a Help button and menu entry open the fork's issue tracker, "This app on GitHub" opens the fork, and "PdfCraft on GitHub" stays as attribution to upstream. The CLI's help lines point at the issue tracker too (`006_pdf-release-v0-1` sprint 2).
- **Replaced** the PdfCraft lion app icon with a contributor-original placeholder (a document and a magnifier on teal) in every size and format (`packaging/icons.sh`). The file names stay upstream's for now (`006_pdf-release-v0-1` sprint 2).
- **Removed** `docs/brand/` (the ArtCraft trademarks) and its attribution entries. `NOTICE` now states that this is a modified version of PdfCraft by Costa Inspection, not made or endorsed by the ArtCraft Team (`006_pdf-release-v0-1` sprint 2).
- **Changed** the packaging publisher to Costa Inspection, with the fork's URLs: the Windows installer (`pdfcraft.wxs`, its MSI test), the Linux package (`nfpm.yaml`), AppStream metadata, and macOS Credits (`006_pdf-release-v0-1` sprint 2).
- **For sprint 5 (needs the suite name, U1):**
  - the app id `ai.storyteller.pdfcraft` is still used in `apps/pdfcraft/src/main.rs` (Wayland app id), `packaging/icons.sh`, `packaging/macos/Info.plist.in`, `packaging/linux/` (desktop, metainfo, flatpak, package scripts), `packaging/freebsd/package.sh`, `crates/ui-egui/src/macos_association.rs`, `xtask/src/gates.rs`, `xtask/src/macos_integration.rs` and the hicolor icon file names. A new app id moves the settings and recovery directory;
  - the user-visible product name "PdfCraft" is still used in installers and metadata;
  - **the MSI `UpgradeCode` (`pdfcraft.wxs`) is upstream's**, so this installer would replace an installed upstream PdfCraft. It needs a new GUID.

- **Fixed** baseline test failures in the fork's earlier work: a quick drag from a crop's corner handle now resizes the crop (the drag start is hit-tested where the drag began, not where the pointer is when egui reports it); the crop OCR-mode dropdown now has an accessible name (labelled by its "OCR" label); the Crops panel has a deterministic snapshot; the panel snapshot test no longer lists the legacy `Metadata` panel (restored as Crops); the fill & sign off-page cursor test is back to upstream's "pointer stays visible" check. Local full test runs need `CRAFT_FONTS_DIR` set to an **absolute** path of a craft-fonts checkout for 10 CJK/Thai tests (`006_pdf-release-v0-1` sprint 1b).

- **Removed** the ArtCraft trademarks from the app: a `branding` module now names the build (`<Suite> PDF Review`, a placeholder until the suite name is chosen), the About dialog and home screen show a contributor-original suite mark, and About states "Based on PdfCraft" in plain text (`006_pdf-release-v0-1` sprint 1).

- **Planned** the first public release of the fork as the Costa Inspection suite's PDF Review tool: project `006_pdf-release-v0-1` (rebrand without the ArtCraft trademarks, portable Windows zip with OCR models, fork docs under `docs/fork/`, release gated on the suite name).

## 2026-10-25 (on top of upstream 0.14.0)

- **Restored** the desktop `edit.snapshot` tool to its original copy-only behavior — dragging a snapshot rectangle now copies pixels without opening the package-export dialog, while crop-review package export and the shared headless snapshot package tool remain available on their own paths.

- **Improved** crop-review snapshot export text extraction — export now prefers native PDF text inside the selected rectangle and falls back to bitmap OCR only when that portion has no usable text layer, which fixes poor `ocr_text` / `report.marker.json` output on born-digital papers while keeping scanned-region behavior available.

- **Aligned** crop-review package export across automation, desktop UI, and docs — the automation tools and desktop export dialog now expose optional `report.docx`, derived `report.marker.json`, and explicit package-folder naming consistently, and the desktop crop-review export flow honors the override destination as the parent folder while still creating the named package directory.

- **Refactored** crop-review mutations behind `pdfcraft-engine` session helpers — the desktop crop-review UI now adds, updates, deletes, and project-loads crop items through explicit engine-owned operations instead of mutating the store directly at each call site, while keeping crop selection and project persistence behavior covered by focused regressions.

- **Hardened** embedded crop-review persistence in `pdfcraft-engine` — focused export-path regressions now cover restoring the reserved `pdfcraft-crop-review.json` attachment on open, preserving it through save/reopen, removing it when the crop store becomes empty, and leaving unchanged documents without crop review free of the reserved attachment.

- **Documented** crop-review persistence shapes and invariants — `docs/crop-review-workspace.md` now states that `source.pdf` plus `project.json` remains the canonical relocatable workspace, and it documents the implemented embedded PDF mirror at reserved attachment name `pdfcraft-crop-review.json` with schema-versioned UTF-8 JSON carrying the persisted crop fields.

- **Documented** the current 1.1 crop-review export contract — `manifest.json` now carries canonical per-item `ocr_text`, the package includes derived `report.marker.json` and optional `report.docx`, and `report.tex` is documented with its shipped minimal `enumerate` + `minipage` structure.

- **Registered** sprint project `docs/changes/005_export-contract-hardening/` — follow-up plan to document and harden the evolved crop-review export contract: canonical OCR in `manifest.json`, derived AI-oriented `report.marker.json`, optional `report.docx`, clarified external vs embedded crop-review persistence, and a later refactor toward engine-owned crop-review mutations.

- **Integrated** crop-review export across engine, automation, and UI — snapshot packages can now be emitted from the persisted crop-review dataset with canonical crop ordering and the existing OCR rule, while the original `edit.snapshot` marquee flow remains a separate transient copy path.

- **Added** crop-review project-folder persistence across engine and UI — `pdfcraft-engine` now writes and reloads self-contained crop-review projects with UTF-8 `project.json` plus copied `source.pdf`, strict schema/path validation, and regression coverage for non-ASCII metadata, while `pdfcraft-ui-egui` can save and reopen those projects back into the Crops/Metadata workspace with overlay state rebuilt faithfully.

- **Added** a dedicated persistent crop-review rectangle tool in `pdfcraft-ui-egui` — the crop workspace now reuses the existing marquee drag mechanics to create stored crop items with stable ids and PDF-space geometry, selects the new crop immediately into the Crops/Metadata flow, and keeps `edit.snapshot` as a separate copy-only path with focused UI non-regression coverage.

- **Added** crop-review metadata and synchronized selection in `pdfcraft-ui-egui` — stored crop rectangles now paint as page overlays, clicks on either the page overlay or the Crops panel drive one selected-crop path, and a dedicated `Metadata` right-side panel edits `caption` and `note` in place with focused UI coverage for both selection directions and stable metadata updates.

- **Added** a dedicated `Crops` right-side panel in `pdfcraft-ui-egui` — the rail, View ▸ Side panels menu, panel dispatcher, and snapshot coverage now expose crop-review items in canonical engine order, with row labels showing `caption` or `Page N` plus `Snapshot · Page N`, and focused UI tests for ordering and row semantics.

- **Added** an engine-owned crop-review model and ordered store in `pdfcraft-engine` — persistent crop items now keep stable ids, canonical PDF-space rectangles, typed metadata, OCR fields, and `chrono` timestamps in per-document state, with canonical `(page, rect_y, rect_x, id)` ordering derived through real page display transforms and focused engine tests for ordering, geometry round-trips, and field preservation.

- **Specified** `docs/crop-review-workspace.md` and cross-referenced `docs/rectangle-driven-export.md` — the repo-local contract now locks the persistent crop-review workspace as separate from the original snapshot tool, with dedicated `Crops` and `Metadata` right-side panels, canonical crop ordering, PDF-space geometry, a self-contained project folder using UTF-8 `project.json` plus copied `source.pdf`, and later export driven from stored crop items.

- **Registered** sprint project `docs/changes/004_crop-review-workspace/` — phased plan for a persistent crop-review workspace with dedicated Crops and Metadata right-side panels, synchronized overlay selection, project-folder persistence, and export driven from stored crop items instead of the original snapshot tool.

- **Specified** `docs/rectangle-driven-export.md` and marked `docs/selected-rectangle-export.md` superseded for `003_export-flow` where the notes conflict — rectangle-driven evidence export now uses user-created rectangles in the shared displayed-page coordinate contract, explicit typed `caption`/`note` metadata, OCR derived from the selected portion into the minimal figure-oriented `report.tex`, and one planned engine path shared by headless and desktop entrypoints.

- **Aligned** the engine snapshot export model with `003_export-flow` — `crates/engine/src/export.rs` now carries explicit rectangle-driven metadata (`caption` plus optional `note`) and an OCR seam on the shared normalized item path, while keeping the current package contract and deterministic manifest/image/report generation unchanged.

- **Added** a headless rectangle-driven snapshot export tool — `pdfcraft-automation` now writes the standard package from ordered page rectangles plus typed `caption`/`note` metadata, reusing the shared engine package writer and preserving root confinement and deterministic item ordering.

- **Added** a desktop rectangle-export flow on top of the existing snapshot marquee tool — dragging a snapshot rectangle now opens a focused package-export dialog for typed `caption`/`note` metadata, then writes the same standard package through the shared engine export path while still copying the selected pixels to the clipboard.

- **Integrated** OCR into the rectangle-driven export flow — snapshot package export now derives OCR text from the selected portion itself, emits it as a derived `OCR.` paragraph in `report.tex`, keeps `manifest.json` unchanged, and follows the repo's existing failure rule when OCR models are unavailable.

- **Registered** sprint project `docs/changes/003_export-flow/` — phased plan for rectangle-driven evidence export built from user-created selections, typed metadata, OCR of the selected portion, and minimal LaTeX package output with headless and desktop parity.
- **Added** an engine bridge from selected rectangle comments to snapshot export items — supported-kind filtering, displayed-rectangle mapping, type-label captions, comment-text notes, and canonical comment-target ordering for reuse by UI and automation.
- **Specified** `docs/selected-rectangle-export.md` — repo-local contract for selected-comment snapshot export covering supported v1 annotation kinds, `caption`/`note` mapping, displayed-rectangle semantics, canonical export ordering, and later UI/automation handoff into the engine.
- **Registered** sprint project `docs/changes/002_selected-rec/` — phased plan for selected rectangle comment export through the current standalone snapshot package/report flow.
- **Hardened** snapshot export v1 with repeatability and guardrail coverage — deterministic package reruns, explicit rejection of non-`snapshot` evidence kinds in v1, and final contributor-facing docs for extension seams.
- **Added** end-to-end snapshot export package writing in `crates/engine/src/export.rs` — emits `manifest.json`, `source.pdf`, `report.tex`, and cropped PNG evidence via staged directory publication with validation and failure cleanup.
- **Added** manifest and LaTeX generation core in `crates/engine/src/export.rs` — deterministic `manifest.json` and `report.tex` output from the shared normalized snapshot item list, with golden coverage for alignment and escaping.
- **Added** a normalized snapshot export item model in `crates/engine/src/export.rs` — deterministic ordering, `item_N` ID assignment, canonical `evidence/<id>.png` paths, and validation for pages, rectangles, and package-relative paths.
- **Specified** `docs/snapshot-export-spec.md` — v1 snapshot export contract with canonical `manifest.json`, deterministic IDs/paths, explicit coordinate and `source.pdf` semantics, validation rules, and worked examples.
- **Documented** snapshot-export discovery decisions in `docs/snapshot-export-discovery.md` — current coordinate/ordering/text-trust/source-PDF/render semantics that later sprints will implement against.
- **Registered** sprint project `docs/changes/001_snapshot-export-contract/` — phased plan for a stable, extensible snapshot export contract with canonical manifest, deterministic LaTeX output, and package validation.
- **Synced** with upstream `storytold/pdfcraft` `main` (merge commit `77509e78`), bringing this
  fork from upstream `0.8.0` to `0.14.0`.
- **Added** `docs/techind/update-fork-and-release.md` — runbook for syncing this fork with
  upstream and cutting a fresh unsigned release (`d1345400`).
- **Registered** the `<upstream-version>-unsigned.<YYYYMMDD>` release-tag methodology in that
  runbook, and switched its sync step to recommend a PR over a direct push to `main`.

## 2026-10-25 (on top of upstream 0.8.0)

- **Added** `.github/workflows/unsigned-release.yml` — automated, unsigned macOS + Windows
  release pipeline (`macos-14` + `windows-latest` + a `release` job using the workflow's own
  `GITHUB_TOKEN`, no personal credentials). Supersedes an earlier throwaway `windows-build.yml`,
  which was deleted (`6ee90a99`).
- **Added** `docs/techind/` — a case study, Marp slide deck, and portability guide written from
  the process of getting the above pipeline working (`a753f133`):
  - `material-curso.md` (PT-BR) — master's-level course material.
  - `apresentacao.md` (PT-BR) — Marp deck.
  - `guia-portabilidade.md` (EN) — how to reuse the pipeline in another repo.
