# 003_export-flow — Loop Protocol

Build a user-facing export flow where exported evidence is driven by a user-created rectangle, typed metadata, and OCR of the selected portion. Keep v1 snapshot-only, keep the package minimal (`manifest.json`, `source.pdf`, image crops, and a minimal LaTeX artifact), and preserve parity between the desktop app and automation/CLI.

# 003_export-flow — Loop Protocol

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
7. Commit in one commit: `<003_export-flow> sprint N: <title>`.
8. When all sprints are checked, mark the project `Status: ready to archive` and
   archive it later with `/sprint-loop archive <project>`.

Status: active

- [x] [Sprint 1 — Product contract and flow definition](./sprint-1.md) — done 2026-10-26
- [x] [Sprint 2 — Engine export model alignment](./sprint-2.md) — done 2026-10-26
- [x] [Sprint 3 — Headless rectangle export flow](./sprint-3.md) — done 2026-10-26
- [ ] [Sprint 4 — Desktop rectangle export flow](./sprint-4.md)
- [ ] [Sprint 5 — OCR integration and final hardening](./sprint-5.md)
