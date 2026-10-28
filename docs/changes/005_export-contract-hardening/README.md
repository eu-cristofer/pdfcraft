# 005_export-contract-hardening — Loop Protocol

Harden and document the evolved crop-review export contract: canonical OCR in `manifest.json`, derived AI-oriented `report.marker.json`, optional `report.docx`, editable package-folder naming in the desktop export flow, and embedded crop-review persistence in saved PDFs, while clarifying which persistence shape is canonical and which is convenience.

# 005_export-contract-hardening — Loop Protocol

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
7. Commit in one commit: `<005_export-contract-hardening> sprint N: <title>`.
8. When all sprints are checked, mark the project `Status: ready to archive` and
   archive it later with `/sprint-loop archive <project>`.

Status: active

- [x] [Sprint 1 — Document export contract 1.1 and AI artifacts](./sprint-1.md) — done 2026-10-28
- [x] [Sprint 2 — Document crop-review persistence shapes and invariants](./sprint-2.md) — done 2026-10-28
- [x] [Sprint 3 — Engine hardening for embedded crop-review persistence](./sprint-3.md) — done 2026-10-28
- [x] [Sprint 4 — Formalize crop-review mutations at the engine boundary](./sprint-4.md) — done 2026-10-28
- [ ] [Sprint 5 — UI, automation, and docs parity sweep](./sprint-5.md)
