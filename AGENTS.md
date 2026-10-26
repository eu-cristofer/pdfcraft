# AGENTS.md

`CLAUDE.md` is the day-to-day playbook. Read it first; keep this file for the repo facts an agent is most likely to miss.

## Start here

- Begin each session with `CLAUDE.md`, especially its startup sequence: `plan/STATUS.md`, `docs/gaps.md`, `docs/target-app-parity.md`, the relevant task in `plan/execution-plan.md`, and the crate README for the area you touch.
- `plan/` is intentionally local-only and gitignored. Do not create commits that depend on `plan/*` changes.
- If `CLAUDE.md` and repo docs disagree with executable config, trust the executable source: `Cargo.toml`, `xtask`, crate manifests, tests, and CI scripts.

## Workspace shape

- This is a Rust workspace (`Cargo.toml`) with three important tops:
  - `crates/*`: product libraries.
  - `apps/pdfcraft`: desktop app.
  - `apps/pdfcraft-cli`: CLI, batch entrypoints, and opt-in MCP server.
- `xtask` is the project automation entrypoint. Prefer `cargo xtask <command>` over ad hoc scripts.
- `vendor/` contains retained provenance copies and real path-patched dependencies. Do not "clean up" vendor entries casually; read `vendor/README.md` first.

## Architecture rules that change implementation choices

- The engine boundary is real: frontends go through `pdfcraft-engine`; do not reach around it from UI or automation code (`crates/engine/README.md`).
- Headless parity is required for user-facing features: if a feature is exposed in the app, it should also be reachable through `crates/automation` / `pdfcraft-cli run`, with an end-to-end automation test.
- Layering is enforced by `cargo xtask layers`: crates below L7 must not depend on egui/winit/eframe/rfd.
- `pdfcraft-render` is still a bootstrap layer over `hayro` + `lopdf`; avoid broad refactors that assume rendering/inspection already live in `cos`/`model`.

## Commands agents are likely to guess wrong

- Prefer scoped checks while iterating, not `--workspace` by default:
  - Format: `cargo fmt --check`
  - Clippy for one crate: `cargo clippy -p <crate> -j 2 --all-targets -- -D warnings`
  - One test: `cargo test -p <crate> -j 2 <test_name> -- --exact`
  - Unit-only one test: `cargo test -p <crate> -j 2 --lib <test_name> -- --exact`
- Project gates live in `xtask`:
  - `cargo xtask layers`
  - `cargo xtask wasm`
  - `cargo xtask assets`
  - `cargo xtask parity [--partial]`
  - `cargo xtask test`
  - `cargo xtask ci` runs the full local gate stack and stops at first failure.
- App entrypoints:
  - Desktop: `cargo run -p pdfcraft -- <file.pdf>`
  - Headless tools: `cargo run -p pdfcraft-cli -- run --script steps.json --root DIR`
  - MCP server: `cargo run -p pdfcraft-cli -- mcp --root DIR`
- Useful diagnostics / generation:
  - `cargo xtask fuzz --time 300`
  - `cargo xtask demo-pdf` (needs Chrome/Chromium)
  - `cargo xtask screenshots`
  - `cargo xtask models` fetches OCR models into `assets/models/`

## Testing and verification quirks

- `cargo xtask test` prefers `cargo nextest` when installed, then runs doctests.
- `cargo xtask parity` validates `parity/acrobat-features.toml` against real commands, tools, and cited tests. Do not mark a feature `shipped` without matching evidence.
- Automation/tool behavior is often best verified through `pdfcraft-cli run` or `pdfcraft-cli ui`, not only unit tests.
- For UI changes, use the opt-in control channel instead of guessing from code paths:
  - launch app with `--control <file>`
  - drive it with `pdfcraft-cli ui --control <file> ...`
- If you fix a crash or panic path, add a small synthetic regression test; this repo treats that as part of the fix, not follow-up work.

## Asset, font, and provenance rules

- Before adding or changing any asset, read `CLAUDE.md` non-negotiables and verify with `cargo xtask assets`.
- Never add Adobe-derived assets or copied GPL/AGPL code.
- Do not commit new font files here. Shared fonts, including Japanese fonts, come from the separate `craft-fonts` checkout via `CRAFT_FONTS_DIR`.
- `ATTRIBUTION.toml` is the source of truth for asset provenance; `ATTRIBUTION.md` is generated.

## Automation / MCP / safety rules

- The MCP server is opt-in only. Do not make the app, CLI, or packaging start it automatically.
- The UI control channel is opt-in only via `--control <file>`. Do not relax its loopback/token model or log the token.
- Tool and CLI path confinement matters. If you touch file access, preserve `--root` / automation root behavior and its refusal semantics.
- Pages and positions in automation are often 1-based and top-left-origin; check `crates/automation/README.md` before changing tool schemas or geometry behavior.

## Repo-specific coding constraints

- No `unsafe` anywhere in workspace code (`Cargo.toml` sets `unsafe_code = "forbid"`).
- `chrono` owns calendar/time conversion. Do not hand-roll date math.
- Root `clippy.toml` bans raw egui checkbox/radio helpers; use the wrappers in `crates/ui-egui/src/widgets.rs`.
- `rustfmt.toml` uses `max_width = 150`; do not reflow files to narrower defaults.

## Environment and local build gotchas

- Rust minimum is 1.95.
- `CRAFT_FONTS_DIR=<checkout>` enables embedded Japanese fonts; code using `pdfcraft_fonts::CRAFT_FONTS` must still work when it is empty.
- For constrained disks, use the dedicated profile documented in `docs/development.md`: `cargo build --profile low-disk ...` / `cargo test --profile low-disk ...`.
- `PDFCRAFT_RENDERER=gl` forces the OpenGL path; unset normally retries wgpu startup with OpenGL fallback.

## Contributor metadata

- Never hand-edit `contributors/contributors.json`.
- Only add or edit the current contributor's own entry in `contributors/people.toml`, and only if they want it.
