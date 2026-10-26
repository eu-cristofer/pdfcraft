# 002_selected-rec — Loop Protocol

Build a user-facing selected-rectangle evidence export flow on top of PdfCraft's snapshot export v1 backend: selected rectangle comments become normalized snapshot items, export works headlessly and in the desktop app, and the current standalone `report.tex` format remains the shipped output.

# 002_selected-rec — Loop Protocol

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
7. Commit in one commit: `<002_selected-rec> sprint N: <title>`.
8. When all sprints are checked, mark the project `Status: ready to archive` and
   archive it later with `/sprint-loop archive <project>`.

Status: active

- [x] [Sprint 1 — Product contract and source mapping](./sprint-1.md) — done 2026-10-26
- [ ] [Sprint 2 — Engine bridge from rectangle comments to snapshot export items](./sprint-2.md)
- [ ] [Sprint 3 — Headless export tool and automation parity](./sprint-3.md)
- [ ] [Sprint 4 — Desktop export action for selected rectangle comments](./sprint-4.md)
- [ ] [Sprint 5 — End-to-end tests and polish](./sprint-5.md)
