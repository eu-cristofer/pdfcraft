# 001_snapshot-export-contract — Loop Protocol

Build a stable, versioned snapshot export package for reporting workflows: canonical `manifest.json`, deterministic cropped images, deterministic LaTeX snapshot blocks, and validation rules that keep the format reproducible now and extensible to future evidence kinds and output targets.

# 001_snapshot-export-contract — Loop Protocol

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
7. Commit in one commit: `<001_snapshot-export-contract> sprint N: <title>`.
8. When all sprints are checked, mark the project `Status: ready to archive` and
   archive it later with `/sprint-loop archive <project>`.

Status: active

- [x] [Sprint 1 — Discovery and contract decisions](./sprint-1.md) — done 2026-10-25
- [x] [Sprint 2 — Spec hardening and examples](./sprint-2.md) — done 2026-10-25
- [x] [Sprint 3 — Internal export model and ordering](./sprint-3.md) — done 2026-10-25
- [ ] [Sprint 4 — Manifest and LaTeX generation core](./sprint-4.md)
- [ ] [Sprint 5 — Raster/source export and validation](./sprint-5.md)
- [ ] [Sprint 6 — Tests, docs, and extensibility guardrails](./sprint-6.md)
