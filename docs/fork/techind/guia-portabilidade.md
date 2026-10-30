# Porting `unsigned-release.yml` to another repository

A practical checklist for reusing this repo's unsigned macOS + Windows release pipeline
(`.github/workflows/unsigned-release.yml`) elsewhere. Background and rationale are in
`material-curso.md` (Portuguese) — this file is just the "how", in English, so it travels to
repos whose contributors or conventions aren't Portuguese.

## What the pipeline assumes

Three independent jobs, triggered by `workflow_dispatch`:

1. **`macos`** (`runs-on: macos-14`) — runs a packaging script that produces a `.dmg` and a CLI
   zip, ad-hoc signed (no certificate needed).
2. **`windows`** (`runs-on: windows-latest`) — a plain `cargo build --release`, zipped.
3. **`release`** (`runs-on: ubuntu-latest`, `needs: [macos, windows]`) — downloads both build
   artifacts and runs `gh release create` using the workflow's own `github.token`. No secrets, no
   personal authentication, no `gh auth login` on anyone's machine.

That shape is generic. What's specific to PdfCraft, and must change, is narrower than it looks.

## Checklist

- [ ] **Binary names.** Replace `pdfcraft` / `pdfcraft-cli` everywhere they appear (the `cargo
      build -p ...` flags, the zip step, the artifact filenames) with your crate's actual binary
      name(s).
- [ ] **The macOS packaging step.** PdfCraft has its own `packaging/macos/package.sh`
      (ad-hoc signs, builds a `.dmg`, defaults to a universal binary). You have three options,
      in order of preference:
  - [ ] If your repo already has an equivalent packaging script (produces a `.app`/`.dmg` or
        just a plain binary), call it the same way — just update the path.
  - [ ] If it doesn't, replace the `macos` job's single "Build and package" step with a plain
        `cargo build --release -p <your-bin>` and skip the `.dmg`/`.app` bundling entirely —
        ship the raw binary, same as the `windows` job does. Simpler, and still a real, working
        build.
  - [ ] If you *do* want a signed `.app`/`.dmg` eventually, that's a separate, deliberate
        decision (see "When to stop being unsigned" below) — don't half-implement signing here.
- [ ] **Rust targets.** If you keep a universal (`x86_64` + `aarch64`) macOS build, make sure
      `dtolnay/rust-toolchain`'s `targets:` input lists both, and that your packaging script (or
      a `lipo -create` step, if you write it inline) actually combines them.
- [ ] **The `release` job: copy as-is.** The download-artifact → `gh release create` shape has
      nothing PdfCraft-specific in it. The only thing you'll typically touch is the release notes
      heredoc (mentions "no Linux/FreeBSD/web build" — adjust to whatever your repo *doesn't*
      build here) and the glob pattern for uploaded assets if you renamed anything above.
- [ ] **Permissions.** Keep `permissions: contents: write` at the workflow level — it's what lets
      `github.token` create a release without a personal access token. Don't broaden it further.
- [ ] **The runner tier.** Don't blindly copy a `runs-on: macos-15-xlarge` (or any other
      large/self-hosted runner label) from somewhere else's workflow into yours unless you've
      confirmed your account/org actually has access to that tier — that's the exact failure
      mode this whole pipeline exists to avoid. `macos-14` / `macos-latest` / `windows-latest` /
      `ubuntu-latest` are available to every account.
- [ ] **Trigger.** `workflow_dispatch` with a `tag` input keeps this decoupled from your repo's
      real release/tag scheme, so it can't collide with an actual version tag. Keep that unless
      you deliberately want this to run on every push to a specific branch — if so, make sure
      that branch isn't the one your *real* release process (if you have one) also watches, for
      the same reason the signing gate in PdfCraft's upstream workflow exists (see
      `material-curso.md`, Lesson 2).

## What you get, honestly

Real, working, **unsigned** binaries:

- macOS: Gatekeeper will block first launch (right-click → Open, or `xattr -dr
  com.apple.quarantine` on the extracted `.app`).
- Windows: SmartScreen will warn on first run ("More info" → "Run anyway").

That's a stated trade-off, not a bug. If your audience is yourself, your team, or early testers
who can be told to override a Gatekeeper/SmartScreen prompt once, this is sufficient and costs
nothing. It is not a substitute for a real release process if you're shipping to the general
public.

## When to stop being unsigned

Revisit this the moment any of the following becomes true:

- You're distributing to people who can't be talked through a security override.
- You need auto-update infrastructure (most auto-updaters refuse unsigned binaries).
- You're submitting to a store or distribution channel that requires signing.

At that point you need real credentials — an Apple Developer Program membership (certificate +
notarization), and either a traditional Windows code-signing certificate or an Azure Trusted
Signing subscription — and the signing logic belongs behind the same kind of hard gate PdfCraft's
own release pipeline uses (`if: github.ref == 'refs/heads/release'` or equivalent), not folded
quietly into this throwaway pipeline.
