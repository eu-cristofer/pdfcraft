# Windows No-Admin Portable Distribution Spec

Last reviewed: 2026-10-28

## Status

This document specifies a Windows distribution path for PdfCraft that works without administrator rights.

The required outcome is a portable `.zip` artifact built in GitHub Actions, with OCR models bundled and no dependency on a machine-wide installer.

## Problem

The current Windows installer path is MSI-based and per-machine. That is not suitable for users who cannot elevate or install into `Program Files`.

The repo already has a supported portable packaging path in `packaging/windows/package.ps1`, but not every GitHub Actions path uses it. In particular, a workflow that zips only `pdfcraft.exe` and `pdfcraft-cli.exe` produces a runnable binary set without the OCR models, which breaks Scan & OCR.

## Goal

Provide a first-class Windows artifact that:

- is built in GitHub Actions on a Windows runner
- runs without admin rights
- does not require MSI installation
- includes the OCR models in the layout PdfCraft already expects
- preserves portable-mode behavior for settings and recovery data
- is publishable as the default Windows artifact for no-admin users

## Non-goals

- adding a per-user MSI installer
- adding Start Menu integration, registry integration, file associations, or shell verbs without admin rights
- changing how PdfCraft discovers OCR models at runtime
- changing the OCR model filenames or provenance flow
- changing the existing MSI behavior

## User story

As a Windows user without administrator rights, I can download one `.zip`, extract it into a folder I can write to, launch `pdfcraft.exe`, and use Scan & OCR without setting `PDFCRAFT_MODELS` manually.

## Requirements

### Distribution requirements

- The no-admin Windows distribution MUST be a `.zip` archive, not an MSI.
- The archive MUST be extractable into a user-writable folder such as `%LOCALAPPDATA%`, `%USERPROFILE%\Downloads`, or `%USERPROFILE%\Desktop`.
- Launching the app from the extracted folder MUST NOT require elevation.
- The archive MUST include `portable.txt` beside `pdfcraft.exe` so PdfCraft runs in portable mode.

### OCR requirements

- The archive MUST include a `models\` directory beside `pdfcraft.exe`.
- That directory MUST contain every OCR model file that `cargo xtask models` stages for Windows packaging.
- At minimum, the staged payload MUST include `text-detection.rten` and `text-recognition.rten`.
- The staged model directory MUST also include the attribution and license files produced by `cargo xtask models`.
- A user extracting the archive and launching PdfCraft MUST NOT need to set `PDFCRAFT_MODELS` manually.

### Packaging requirements

- GitHub Actions MUST build the Windows no-admin artifact by invoking `packaging/windows/package.ps1`, not by zipping raw executables directly.
- The workflow MUST publish the generated `*-portable.zip` artifact.
- If the release channel is intended for no-admin users, that portable artifact SHOULD be the primary Windows download.
- The packaging job MUST retain network access for `cargo xtask models`.

### Compatibility requirements

- The portable artifact MUST keep the existing runtime layout contract used by `pdfcraft_ocr::Models::dirs_beside_exe`.
- The portable artifact MUST keep the existing portable-mode contract used by `crates/ui-egui/src/portable.rs`.
- The implementation MUST NOT introduce a second Windows portable layout for OCR.

## Package contract

The extracted portable folder MUST have this layout:

```text
<pdfcraft-portable>/
  pdfcraft.exe
  pdfcraft-cli.exe
  portable.txt
  README.md
  LICENSE
  LICENSE-MIT
  LICENSE-APACHE
  NOTICE
  LICENSE-document.txt
  models/
    text-detection.rten
    text-recognition.rten
    ATTRIBUTION.txt
    *.LICENCE.txt
  PdfCraftData/            # created at runtime, not shipped in the zip
```

Notes:

- `PdfCraftData/` is runtime output and MUST NOT be pre-created in the release archive.
- Additional packaged files are allowed if `packaging/windows/package.ps1` already stages them, but the layout above is the minimum supported contract for the no-admin distribution.

## Implementation plan

### 1. Standardize on the existing Windows packaging script

The GitHub Actions Windows distribution path MUST call:

```powershell
pwsh packaging/windows/package.ps1 -Arch x64
```

Reason:

- it stages `portable.txt`
- it stages the `models\` folder
- it runs `cargo xtask models <dest>`
- it produces the final portable zip using the supported layout

No workflow that intends to ship OCR support may replace this with `Compress-Archive` over bare executables.

### 2. Update the GitHub Actions Windows artifact selection

Any repo-owned GitHub Actions workflow that currently uploads only a hand-built zip of `.exe` files MUST be changed to upload the portable zip produced by `package.ps1`.

Required artifact pattern:

```text
dist/release/*-windows-x64-portable.zip
```

Optional:

- upload the MSI as a separate artifact for admin-capable users
- keep the MSI out of the no-admin release channel entirely

### 3. Preserve OCR staging in CI

The Windows CI packaging job MUST not strip the `models\` directory before zipping or publishing.

The job MUST fail if OCR staging did not produce:

- `ATTRIBUTION.txt`
- at least one `*.LICENCE.txt`
- the expected model binaries

This failure may continue to rely on the existing checks in `package.ps1`; no parallel implementation should be added unless there is a demonstrated gap.

### 4. Document the user-facing install model

The Windows release notes and any download instructions SHOULD say:

- extract the zip to a writable folder
- launch `pdfcraft.exe`
- no administrator rights are required
- OCR is included in the portable package

The docs SHOULD NOT describe the portable zip as an installer.

## GitHub Actions changes

### Minimum workflow behavior

A compliant Windows GitHub Actions job MUST:

1. run on `windows-latest` or another supported Windows runner
2. check out the repo
3. install the Rust toolchain
4. install WiX v5
5. run `pwsh packaging/windows/package.ps1 -Arch x64`
6. upload `dist/release/*-windows-x64-portable.zip`

### Recommended workflow behavior

- Upload the MSI separately only if that release channel intends to support admin installs.
- Name the artifact so users can distinguish it from installer-based Windows releases.
- Keep the portable zip as the Windows asset referenced in no-admin documentation.

### Shared-workflow note

The repo's `release.yml` delegates to a shared workflow outside this checkout. If that shared workflow does not already use `packaging/windows/package.ps1`, implementation can proceed in either of these ways:

1. update the shared release workflow to use the repo's packaging script for Windows
2. add or retain a repo-owned Windows packaging workflow for the no-admin channel

The required invariant is the artifact contract, not the exact workflow ownership model.

## Verification

### CI verification

The Windows packaging job MUST verify that the produced portable archive contains:

- `pdfcraft.exe`
- `pdfcraft-cli.exe`
- `portable.txt`
- `models/text-detection.rten`
- `models/text-recognition.rten`
- `models/ATTRIBUTION.txt`

This verification can be done either before zipping, after zipping, or both.

### Runtime verification

At least one Windows acceptance check MUST unzip the produced portable artifact into a normal user-writable directory and verify:

1. `pdfcraft.exe` starts without elevation
2. PdfCraft runs in portable mode
3. OCR status reports models available
4. no manual `PDFCRAFT_MODELS` environment variable is needed

If a UI-level automated verification is too heavy for the first pass, a CLI or automation-based check that confirms OCR availability is acceptable.

## Acceptance criteria

This work is complete when all of the following are true:

1. GitHub Actions publishes a Windows portable zip built through `packaging/windows/package.ps1`.
2. The published zip contains the `models\` folder beside `pdfcraft.exe`.
3. A user without administrator rights can extract and run the app from a writable folder.
4. Scan & OCR works from that extracted portable build without setting `PDFCRAFT_MODELS`.
5. The no-admin documentation points users to the portable zip, not the MSI.

## Risks

- `cargo xtask models` needs network access, so Windows packaging becomes sensitive to model-host availability.
- If a workflow silently reverts to direct `Compress-Archive`, OCR will regress again for portable users.
- If release messaging is ambiguous, users without admin rights may still download the MSI first.

## Follow-up work

- add an archive-content test in CI that inspects the published portable zip
- add an automation smoke test for OCR availability on the packaged artifact
- decide whether the main Windows release page should prefer the portable zip or present both artifacts equally
