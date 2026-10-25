# Updating this fork and cutting a fresh unsigned release

A runbook for the recurring situation: upstream (`storytold/pdfcraft`) has moved, and you want
this fork (`eu-cristofer/pdfcraft`) caught up and a new unsigned macOS + Windows build published
from the current code. Written to be followed exactly either by a human or by an agent — every
command is copy-pasteable, and every step that needs a judgment call says so explicitly.

Background on *why* this pipeline exists and what it trades off: `material-curso.md` (the case
study) and `guia-portabilidade.md` (porting it elsewhere) in this same folder.

## 1. One-time setup: add the upstream remote

Only needed once per clone/machine — check first, it may already be there:

```sh
git remote -v
```

If `upstream` isn't listed:

```sh
git remote add upstream https://github.com/storytold/pdfcraft.git
```

## 2. Sync this fork's `main` with upstream

This fork's `main` is **not** a pure mirror — it carries commits upstream doesn't have
(`.github/workflows/unsigned-release.yml`, `docs/techind/`). That means a fast-forward won't
work; this is a real merge.

```sh
git checkout main
git fetch origin
git merge --ff-only origin/main        # make sure local main matches your fork before syncing
git fetch upstream
git merge upstream/main                # NOT --ff-only — this fork has its own commits
```

**Stop and look if this reports conflicts.** Resolve them by hand, read every conflicting hunk
(not just the ones git couldn't auto-merge — this repo's own `CLAUDE.md` makes the same point
about rebases, and it applies here too: "`main` moves fast enough that a file can change by
hundreds of lines under a branch"), and only continue once you understand what each side changed.
An agent should **not** auto-resolve a conflict by blindly picking one side.

**Recommended: land it through a PR, not a direct push to `main`.** This repo's own `CLAUDE.md`
states the convention plainly — "`main`... is never pushed to directly... open a PR" — and a
sync is exactly the kind of change worth a second look before it lands, even on a fork:

```sh
git checkout -b sync/$(date +%Y%m%d)
git push origin sync/$(date +%Y%m%d)
# then open the PR GitHub prints a link for, and merge once it's green
```

If you'd rather push straight to `main` (e.g. a solo fork, no CI gating it), that still works —
`git push origin main` — but treat it as the fallback, not the default.

## 3. Sanity check before releasing (recommended)

Upstream moves fast — this repo's own `CLAUDE.md` notes 396 commits in three days at one point.
A quick local check catches an obvious break before you spend CI minutes on it:

```sh
cargo check -p pdfcraft -p pdfcraft-cli
```

If this fails, stop here — fix or investigate before releasing, don't publish a build you know is
broken.

## 4. Cut a fresh release

**Tag methodology: `<upstream-version>-unsigned.<YYYYMMDD>`** — e.g. `0.14.0-unsigned.20261025`.
Don't reuse a tag — GitHub rejects a release tag that already exists, and this fork's version
number won't necessarily have moved even though the code did, so the date component is what
keeps repeated releases unique. The upstream-version component is what makes the tag self-
describing: looking at a release tells you which `storytold/pdfcraft` version it was built from,
with no need to cross-reference `CHANGELOG.md`.

```sh
UPSTREAM_VERSION=$(awk '
  /^\[/ { in_pkg = ($0 == "[workspace.package]"); next }
  in_pkg && $1 == "version" { gsub(/[" ]/, "", $3); print $3; exit }
' Cargo.toml)   # same one-liner packaging/env.sh uses to read the version
TAG="${UPSTREAM_VERSION}-unsigned.$(date +%Y%m%d)"
```

If you cut two releases from the *same* upstream version on the *same* day, append the short
commit SHA to disambiguate: `TAG="${TAG}.$(git rev-parse --short HEAD)"`.

**Web UI:**
1. `https://github.com/eu-cristofer/pdfcraft/actions/workflows/unsigned-release.yml`
2. "Run workflow" → branch `main` → tag input: the `$TAG` value from above → "Run workflow"

**`gh` CLI** (needs `gh auth login` once, interactively — an agent cannot complete that part):

```sh
gh workflow run unsigned-release.yml --repo eu-cristofer/pdfcraft -f tag="$TAG"
gh run watch --repo eu-cristofer/pdfcraft   # follow the run live; exits non-zero if it fails
```

## 5. Verify

```sh
gh release view "$TAG" --repo eu-cristofer/pdfcraft
```

Expect four assets: the macOS `.dmg`, the macOS CLI `.zip`, and the Windows build's files —
same shape as the first release. The `macos` job's large-runner/signing caveats from the case
study don't apply here; this pipeline never touches either.
