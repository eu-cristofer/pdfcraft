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
