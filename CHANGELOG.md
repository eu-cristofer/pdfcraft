# Changelog (this fork only)

`eu-cristofer/pdfcraft` is a fork of [`storytold/pdfcraft`](https://github.com/storytold/pdfcraft).
This file tracks only the changes made **in this fork**, on top of whatever upstream version was
current at the time — it is not, and does not try to be, upstream's own changelog or release
history. Entries are keyed by date and the upstream version they were built on, since the version
number in `Cargo.toml` is upstream's, not this fork's, to assign meaning to.

See `docs/techind/update-fork-and-release.md` for the sync/re-release process these entries come
from. Release tags follow that same runbook's methodology,
`<upstream-version>-unsigned.<YYYYMMDD>` (e.g. `0.14.0-unsigned.20261025`) — so a release tag and
a changelog entry always describe the same upstream lineage, just in two different places.

## 2026-10-25 (on top of upstream 0.14.0)

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
