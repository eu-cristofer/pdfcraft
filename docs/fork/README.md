# Fork documentation

Status: living document · Last reviewed: 2026-10-29

This repository is Costa Inspection's fork of PdfCraft, shipped as "<Suite> PDF Review". It adds a crop-review workspace and an evidence-package export on top of upstream. Fork-only documentation lives here, so that upstream-owned files stay as close to upstream as possible.

## Contents

| File | Description |
|---|---|
| [crop-review-workspace.md](crop-review-workspace.md) | Crop-review workspace contract |
| [snapshot-export-spec.md](snapshot-export-spec.md) | Evidence package v1.1 contract (normative) |
| [snapshot-export-discovery.md](snapshot-export-discovery.md) | Discovery decisions behind the export contract |
| [rectangle-driven-export.md](rectangle-driven-export.md) | Rectangle-driven export flow; source of truth for 003 |
| [selected-rectangle-export.md](selected-rectangle-export.md) | Historical: selected-comment export, superseded |
| [fork-sync-sop.md](fork-sync-sop.md) | Keeping the fork in sync with upstream |
| [SOP/](SOP/) | SOP folder |
| [techind/](techind/) | Course material and portability guide (pt-BR) |
| [to-do-later.md](to-do-later.md) | Windows no-admin portable distribution spec (implemented) |
| [agent-notes.md](agent-notes.md) | The fork's earlier agent guide, kept for reference (upstream's `AGENTS.md` is restored at the root) |

## Sprint projects

Active projects are in [../changes/](../changes/); finished ones are in [../archive/](../archive/) (sprint-loop layout).

## Working notes

- Run one test: `cargo test -p <crate> -j 2 <test_name> -- --exact` (add `--lib` to skip integration tests when the test is unit-level).
- Full UI test runs need `CRAFT_FONTS_DIR` set to an absolute path of a craft-fonts checkout; without it, 10 CJK/Thai tests fail with "No glyph".
- One-time per clone: `git config merge.ours.driver true`, so that `.gitattributes`'s `README.md merge=ours` keeps the fork's README on upstream merges.
- Upstream is a git remote named `upstream` (https://github.com/storytold/pdfcraft); see [fork-sync-sop.md](fork-sync-sop.md).

## Brand rule

The ArtCraft marks (name, wordmark, logos) belong to the ArtCraft Team and must never be added back: no logos, no Discord or community invitations, no "part of ArtCraft" wording. The plain-text line "Based on PdfCraft by the ArtCraft team" is allowed. Names and ids of this build live in `crates/ui-egui/src/branding.rs`.
