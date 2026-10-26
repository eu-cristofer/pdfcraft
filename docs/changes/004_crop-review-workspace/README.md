# 004_crop-review-workspace — Loop Protocol

Build a persistent crop-review workspace inside PdfCraft's existing document shell: a dedicated rectangle-annotation workflow with right-side Crops and Metadata panels, synchronized overlay selection, project persistence, and export driven from the stored crop dataset rather than from the original snapshot tool.

# 004_crop-review-workspace — Loop Protocol

Each run MUST:
1. Read this README top to bottom. Pick the FIRST sprint whose checkbox is `[ ]`.
    Never skip ahead; never start a sprint whose predecessor is unchecked.
2. Open its sprint-N.md and verify the "Assumes" line against the code. If an
   assumption fails, fix forward only if trivial; otherwise write
   `BLOCKED: <reason>` under the tracker line and STOP (do not tick).
3. Implement exactly the sprint's scope (via `/sprint-loop next <project>` or
   `/sprint-loop run <project>`: the hosted drafter drafts, the orchestrator
   reviews/applies). Resist scope creep.
4. Run the sprint's Acceptance commands — all must pass.
5. Update the changelog (one bullet per user-visible change).
6. Tick the checkbox `[x]` and append `— done YYYY-MM-DD`.
7. Commit in one commit: `<004_crop-review-workspace> sprint N: <title>`.
8. When all sprints are checked, mark the project `Status: ready to archive` and
   archive it later with `/sprint-loop archive <project>`.

Status: active

- [x] [Sprint 1 — Workspace contract and product boundaries](./sprint-1.md) — done 2026-10-26
- [x] [Sprint 2 — Crop model and ordered store](./sprint-2.md) — done 2026-10-26
- [x] [Sprint 3 — Crops right-side panel](./sprint-3.md) — done 2026-10-26
- [x] [Sprint 4 — Metadata panel and selection synchronization](./sprint-4.md) — done 2026-10-26
- [x] [Sprint 5 — Rectangle creation tool and overlay persistence](./sprint-5.md) — done 2026-10-26
- [x] [Sprint 6 — Project-folder persistence lifecycle](./sprint-6.md) — done 2026-10-26
- [ ] [Sprint 7 — Export integration, OCR, and final hardening](./sprint-7.md)
