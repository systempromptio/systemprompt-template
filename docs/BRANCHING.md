# Branching: `next` and `main`

Two branches. `next` is where everything lands; `main` is what people install.
Which core a branch builds against is the only thing that differs between
them, and the tooling below keeps that honest.

| | `next` | `main` |
|---|---|---|
| Purpose | development; the coming release | released; every commit on it is a promotion merge with a `vX.Y.Z` tag |
| Default branch | yes — a fresh clone starts here | protected by a ruleset: pull request only, no bypass |
| Core | the published release, **or** a core commit pinned by git rev (every core entry a `{ version, git, rev }` dependency in **both** `Cargo.toml` and `tests/Cargo.toml`, no `[patch]`) while the next core is unreleased | always the published release |
| Local core override | `scripts/core-patch.sh on <checkout>` in the working tree only; never committed (`.githooks/pre-commit`) | never |
| `bridge/CORE_REF` | `vX.Y.Z` of the published core, or the 40-char SHA every pin entry names (`just core-pin`) | `vX.Y.Z`, equal to the pin (`scripts/check-core-ref.sh`) |
| CI on push | `.github/workflows/gates.yml` — the whole gate, nothing published | `docker.yml` re-verifies the merge and publishes `:edge` / `:sha-*`; `helm.yml`; `coverage.yml` measures |
| Releases | — | the `vX.Y.Z` tag `just release` pushes at the merge starts `release-gateway.yml` |
| Allowed to break? | a red gate on `next` is information, not an incident | no |

`main` only moves through a frozen promotion PR opened by `just release X.Y.Z`
(see [RELEASING.md](RELEASING.md)). Nothing is committed to `main` directly.

## Lockstep versions

This repo's version is core's version: core `X.Y.Z` on crates.io → workspace
`version = "X.Y.Z"` → tag `vX.Y.Z` → image `:X.Y.Z` → Helm `appVersion`.
`scripts/sync-release-version.sh` owns the product pins (workspace version,
Helm chart, CasaOS / DigitalOcean deploy pins, operator-doc literals) and its
`--check` refuses a release whose `MAJOR.MINOR` differs from the core pin's.
`scripts/sync-core-version.sh` owns the core crate pins in both workspaces
(`systemprompt`, `-security`, `-users`, `-content`, `-marketplace`,
`-extension`, `-api`, `-evaluation`; `extensions/web` inherits
`-extension` from the workspace) and `bridge/CORE_REF`, with a residual sweep
that fails on any `systemprompt*` pin it does not move. `just core-bump X.Y.Z`
runs both. A template-only patch (`0.61.1` on core `0.61.0`) is the one
sanctioned divergence.

## Two lockfiles, one core

`tests/` is a separate cargo workspace with its own `Cargo.lock`, so
`cargo update -w` in the root refreshes only one of them. Both must resolve
every `systemprompt-*` crate at **the same version** — a path copy and a
registry copy may coexist, never two versions, or the test build carries a
second copy of a shared crate and fails with a type mismatch that names
neither lockfile. `scripts/check-core-crate-versions.sh` (in
`just preflight-static`, the `static` tier) enforces it; `just core-bump`
refreshes both.

## The silent-drop trap

A `[patch]` whose version does not satisfy the pin is **dropped without an
error**: cargo resolves the published crate instead, the build passes, and it
has proved nothing about the core you meant to test. `[patch]` is also
per-workspace. That is why the committed pin is not a patch at all: each core
crate the manifests name is a git dependency on the rev, and the crates core
itself depends on resolve inside the same git source. The only `[patch]` is
the working-tree override `scripts/core-patch.sh on` writes.

Defences, in the order they catch it:

1. `scripts/sync-core-version.sh <core-version> --check` — every core pin in
   both workspaces agrees.
2. `scripts/check-core-ref.sh` (a lint gate) — without a core git dependency,
   `bridge/CORE_REF` must be `v<pin>`; with one, `scripts/core-pin.sh --check`
   requires every entry and both lockfiles to name the CORE_REF rev.
3. The lockfiles. With the pin, every core crate's `source` is
   `git+https://github.com/systempromptio/systemprompt-core?rev=<sha>#<sha>`,
   once; `core-pin.sh --check` fails on a registry copy or a second copy.
4. `scripts/validate-release.sh` refuses a release whose manifests carry a
   core git dependency, a `[patch]` table or a `../systemprompt-core` path.

## Working on `next`

Against the published core (the usual state):

```bash
just verify                      # what gates.yml runs: static, lint, test tiers
git push origin next             # gates.yml runs; nothing is published
```

Against an unreleased core commit (pinned by git rev):

```bash
# Push the core commit first; the pin must name a commit on GitHub.
just core-pin <sha|branch>       # bridge/CORE_REF, every core entry in both manifests, both lockfiles
just verify
git push origin next             # CI builds from the pin; no core checkout is needed to compile
```

Against a local core checkout (working tree only):

```bash
scripts/core-patch.sh on ../systemprompt-core   # [patch."<core url>"] path entries; manifests and locks hidden from git
just verify
scripts/core-patch.sh off                       # the committed pin is back
```

`.githooks/pre-commit` (`just init-hooks`) refuses a staged `[patch]` path entry and a
pin that disagrees with `bridge/CORE_REF`, so a local override cannot leak into
a commit; `just deploy` (`core-guard`) refuses while one is on.

Core changes are **write-only from here**: commit them on core's `next` and
push — core's CI is their validation surface. `bridge/CORE_REF` must name a
commit that exists on GitHub, so push core before pushing here.

**Do not run `just prepare` while a local override is on.** It bakes core's own
queries into this repo's `.sqlx/`.

## Landing `next` on `main` (once core X.Y.Z is on crates.io)

```bash
# Drop the git and rev keys from every core entry in BOTH manifests, then:
just core-bump X.Y.Z             # every pin + CORE_REF, both lockfiles, local migrate, build, clippy
just schema-baseline             # record tests/fixtures/schema/release-baseline-X.Y.Z.sql
# write the CHANGELOG `## [X.Y.Z] - YYYY-MM-DD` entry
just verify
git push origin next             # the full Gates matrix runs on this exact commit
just release X.Y.Z               # opens the frozen promotion PR
just release X.Y.Z               # after its proof is green: merges and tags vX.Y.Z
```

`just release` refuses when the tree is dirty, when run off `next` (a clean
detached worktree at `origin/next` also qualifies), when a core git dependency
or a `[patch]` table is present, when a pin or `CORE_REF` disagrees, when the schema ladder has no rung
for the version, when the changelog has no heading for it, when `HEAD` is not
`origin/next`, or when `main` is not an ancestor of it.

Afterwards, to resume development against the next unreleased core, add the
`git` and `rev` keys back to every core entry in both manifests and `just core-pin <sha>`.
