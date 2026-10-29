# Sprint 4 — Fork documentation layout

## Goal

Move fork-only documentation under `docs/fork/` and restore upstream-owned top-level docs to their upstream text plus one pointer line. This reduces future merge conflicts (prep for P5) and makes the fork's own docs easy to find.

## Files

- **Create `docs/fork/README.md`:** index of fork docs. One paragraph on what the fork is (the `<Suite>` PDF Review by Costa Inspection, based on PdfCraft), a link to the suite repo `eu-cristofer/inspection-suite`, and a list of the moved docs.
- **`git mv` into `docs/fork/`:**
  - `docs/crop-review-workspace.md`;
  - `docs/snapshot-export-spec.md`;
  - `docs/snapshot-export-discovery.md`;
  - `docs/selected-rectangle-export.md`;
  - `docs/rectangle-driven-export.md`;
  - `docs/fork-sync-sop.md`;
  - `docs/SOP/`;
  - `docs/techind/`;
  - `docs/to-do-later.md`;
  - the Windows no-admin portable spec (find it with `grep -rln "No-Admin Portable" docs`).
  Confirm each is fork-only before moving: `git log --format=%an 83ba2868..HEAD -- <file>` shows only fork commits, and `git cat-file -e 83ba2868:<file>` fails.
- **Fix every relative link** to the moved files (grep across `docs/`, `crates/*/README.md`, `CHANGELOG.md`, `docs/changes/`, `docs/archive/`). **Do not** edit links inside `docs/archive/*`: archived history may keep stale links, but list them in the commit body.
- **Restore upstream text in `README.md`, `CLAUDE.md` and `ROADMAP.md`:** `git checkout 83ba2868 -- README.md CLAUDE.md ROADMAP.md`, then add one line to `README.md` near the top: "> This is the Costa Inspection fork. Fork-specific docs: [docs/fork/](docs/fork/README.md)."
  - Before restoring `CLAUDE.md`, move its fork-only additions (e.g. the "One test:" bullet) into `docs/fork/README.md` under "Working notes".
- **Sprint folders `docs/changes/002_selected-rec` and `003_export-flow`** (both "Status: active"):
  - check their trackers;
  - if every sprint is `[x]`, mark them `Status: ready to archive`; if not, mark them `Status: superseded by 004/005`, with a one-line reason.
  - Then archive them, and `005_export-contract-hardening`, by `git mv` to `docs/archive/<today>_NNN_slug/` (the `/sprint-loop archive` rules).

## Reuse

- The `/sprint-loop archive` procedure (`git mv` to `docs/archive/YYYY-MM-DD_NNN_slug/`).

## Tests

- Link check, which must print nothing:
  ```bash
  grep -rnoE '\]\((\.\./|\./)?[^)#]+\.md' docs/fork docs/changes README.md CHANGELOG.md crates/*/README.md \
    | while IFS=: read -r f _ m; do t=$(dirname "$f")/$(echo "$m" | sed -E 's/^\]\(//'); [ -e "$t" ] || echo "BROKEN $f -> $t"; done
  ```

## Acceptance

```bash
git diff --stat 83ba2868 -- README.md CLAUDE.md ROADMAP.md   # README +1/+2 lines; CLAUDE.md and ROADMAP.md unchanged
test -f docs/fork/README.md
timeout 1200 cargo test -p pdfcraft-engine -j 2               # in case tests include_str! any moved doc
```
Plus the link check above, which must print nothing.

## Definition of done

- Fork docs live under `docs/fork/`.
- Upstream top-level docs differ from upstream by only the pointer line.
- Projects 002, 003 and 005 are archived.
- The CHANGELOG has a bullet.
- The tracker is ticked.

## Assumes

- Sprints 1–3 are on this branch.
- Upstream fork point `83ba2868` exists locally: `git cat-file -e 83ba2868`.
