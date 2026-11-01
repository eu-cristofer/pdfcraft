# Sprint 3 — Portable Windows zip with OCR models

## Goal

The unsigned release workflow's Windows artifact becomes the portable zip from `packaging/windows/package.ps1`. That zip includes `models\` (OCR) and `portable.txt`, and runs without admin rights.

## Files

- **`.github/workflows/unsigned-release.yml`:**
  - In the Windows job, replace the `Build` and `Zip` steps (`cargo build --release -p pdfcraft -p pdfcraft-cli`, then `Compress-Archive` of the two exes) with a call to `packaging/windows/package.ps1`. Set the environment so that signing and the MSI are skipped and only `*-portable.zip` is produced.
  - Read `package.ps1` first:
    - its inputs are `PDFCRAFT_VERSION`, `PDFCRAFT_BUILD_SHA`, `PDFCRAFT_BUILD_DATE` and `PDFCRAFT_REQUIRE_WINRES`, plus signing variables;
    - its outputs are `pdfcraft-<version>-windows-<arch>.msi` and `…-portable.zip`.
  - If the script cannot skip the MSI or signing, add the **smallest** opt-out switch (e.g. `-PortableOnly`, which skips WiX and signing), defaulting to the current behaviour.
  - Upload `dist/**/*-portable.zip` (or the script's actual output dir) as the `windows-build` artifact.
- **The OCR models:** check how `package.ps1` obtains `models\`, i.e. whether they are downloaded or built by a prior step. If the official `release.yml` has a preceding step that fetches them, copy that step (with the same pinned action SHAs and checksums) into the unsigned workflow.
- **Release notes heredoc** in the same workflow:
  - replace "plain `.exe`s in the zip" with "portable zip: extract anywhere, no admin rights needed; settings stay next to the exe (`portable.txt`)";
  - title: "`<Suite>` PDF Review — unsigned build (`$TAG`)".
- **The fork's no-admin portable spec:** mark its status as implemented and link this sprint. Find it with `grep -rln "No-Admin Portable" docs`.

## Reuse

- `packaging/windows/package.ps1` (portable layout: `pdfcraft.exe`, `pdfcraft-cli.exe`, `models\`, `portable.txt`).
- `packaging/windows/portable.txt`.
- Model-fetch steps from `.github/workflows/release.yml`, if present.

## Tests

- Dispatch the workflow manually with tag `0.14.0-unsigned.<YYYYMMDD>-rc1` from this branch: `gh workflow run unsigned-release.yml -f tag=... --ref feat/006-pdf-release-v0-1`. Delete the rc release afterwards, or keep it as a pre-release.
- Download the Windows artifact and list its contents (`unzip -l`). It must contain `pdfcraft.exe`, `pdfcraft-cli.exe`, `portable.txt` and `models/` with the OCR model files.

## Acceptance

- The workflow run is green.
- The zip listing shows the four required entries. Paste the listing into the CHANGELOG bullet, or into the commit body if long.
- `actionlint` passes on the workflow, if installed; otherwise the `packaging-lint.yml` checks.

## Definition of done

- The Windows artifact is the portable zip with OCR models.
- The release notes are updated.
- The spec is marked implemented.
- The CHANGELOG has a bullet.
- The tracker is ticked.

## Assumes

- Sprints 1–2 are merged on this branch.
- `gh` is authenticated as `eu-cristofer`, with `workflow` scope.
- The branch is pushed to `origin` (required to dispatch the workflow).
