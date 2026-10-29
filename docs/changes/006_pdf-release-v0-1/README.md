# 006_pdf-release-v0-1

Publishes this fork's PDF Review tool (crop review, OCR, evidence package export), exactly as it works on `feat/crop-panel-unification` @ 17649c1b. The release goes out under the suite's own name, with **no ArtCraft trademarks**, and the Windows build is a portable zip that includes the OCR models.

This is gate **G1** of the Costa Inspection suite plan: `eu-cristofer/inspection-suite`, `docs/plan/official-plan.md`, with source spec `docs/plan/projects/P1-pdf-release-v0.1.md`. Out of scope: syncing with upstream (that's P5), new features, signing.

# 006_pdf-release-v0-1 — Loop Protocol

Each run MUST:
1. Read this README top to bottom. Pick the FIRST sprint whose checkbox is `[ ]`.
   Never skip ahead; never start a sprint whose predecessor is unchecked.
2. Open its sprint-N.md and verify the "Assumes" line against the code. If an
   assumption fails, fix forward only if trivial; otherwise write
   `BLOCKED: <reason>` under the tracker line and STOP (do not tick).
3. Implement exactly the sprint's scope (via `/sprint-loop next <project>` or
   `/sprint-loop run <project>`: the drafter drafts, the orchestrator
   reviews/applies). Resist scope creep.
4. Run the sprint's Acceptance commands — all must pass.
5. Update the changelog (one bullet per user-visible change).
6. Tick the checkbox `[x]` and append `— done YYYY-MM-DD`.
7. Commit in one commit: `<006_pdf-release-v0-1> sprint N: <title>`.
8. When all sprints are checked, mark the project `Status: ready to archive` and
   archive it later with `/sprint-loop archive <project>`.

Status: active

- [x] [Sprint 1 — Branding module and removal of the in-app marks](./sprint-1.md) — done 2026-10-29
- [ ] [Sprint 2 — App icon, ids and packaging metadata](./sprint-2.md)
- [ ] [Sprint 3 — Portable Windows zip with OCR models](./sprint-3.md)
- [ ] [Sprint 4 — Fork documentation layout](./sprint-4.md)
- [ ] [Sprint 5 — Release (gated on the suite name, U1)](./sprint-5.md)
