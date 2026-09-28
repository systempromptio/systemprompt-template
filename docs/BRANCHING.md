# Branching: `next` and `main`

Two branches. `next` is where everything lands; `main` is what people install.
Which core a branch builds against is the only thing that differs between
them, and the tooling below keeps that honest.

| | `next` | `main` |
|---|---|---|
| Purpose | development; the coming release | released; every commit on it is a promotion merge with a `vX.Y.Z` tag |
| Default branch | yes — a fresh clone starts here | protected by a ruleset: pull request only, no bypass |
| Core | the published release, **or** core `next` through the sibling `../systemprompt-core` while the next core is unreleased | always the published release |
| Core override | inert under `[workspace.metadata.unreleased-core-patch]`, or live as `[patch.crates-io]` (with the `# ACTIVE: core X.Y.Z is unreleased` marker) in **both** `Cargo.toml` and `tests/Cargo.toml` | inert |
| `bridge/CORE_REF` | `vX.Y.Z` of the pinned core; a 40-char core `next` SHA while the patch is live (`just core-pin`) | `vX.Y.Z`, equal to the pin (`scripts/check-core-ref.sh`) |
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
`-extension`, `-api`, `-evaluation`, the bare-string pin in
`extensions/web/Cargo.toml`) and `bridge/CORE_REF`, with a residual sweep
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
has proved nothing about the core you meant to test. For a 0.x crate
`version = "0.61.0"` means `>=0.61.0, <0.62.0`, so a patch pointing at a
0.62.0 tree is ignored by a 0.61.0 pin. `[patch]` is also per-workspace: patch
the root alone and the test crates quietly build against crates.io.

Defences, in the order they catch it:

1. `scripts/sync-core-version.sh <core-version> --check` — every core pin in
   both workspaces agrees.
2. `scripts/check-core-ref.sh` (a lint gate) — with the patch inert,
   `bridge/CORE_REF` must be `v<pin>`.
3. The build log. With the patch live it must name
   `../systemprompt-core/...` paths for the `systemprompt-*` crates; a bare
   `v0.62.0` with no path is a dropped patch.
4. `scripts/validate-release.sh` refuses a release whose manifests carry a
   live patch or a `../systemprompt-core` path.

## Working on `next`

Against the published core (the usual state):

```bash
just verify                      # what gates.yml runs: static, lint, test tiers
git push origin next             # gates.yml runs; nothing is published
```

Against unreleased core `next`:

```bash
# Rename [workspace.metadata.unreleased-core-patch] to [patch.crates-io] in
# Cargo.toml AND tests/Cargo.toml, add "# ACTIVE: core X.Y.Z is unreleased"
# above each, and move the pins to the sibling's workspace version.
scripts/sync-core-version.sh X.Y.Z --check   # after editing the pins
just core-pin                    # bridge/CORE_REF = the sibling's HEAD; commit it
just verify
git push origin next             # CI checks core out at CORE_REF (.github/actions/core-checkout)
just deploy                      # runs core-guard: the sibling must be clean and at CORE_REF
```

`.githooks/pre-commit` (`just init-hooks`) blocks a live patch without the
marker, so a personal local pin cannot leak into a commit. For a purely local
experiment, keep the patch as a working-tree edit hidden with
`git update-index --skip-worktree Cargo.toml Cargo.lock tests/Cargo.toml tests/Cargo.lock`.

Core changes are **write-only from here**: commit them on core's `next` and
push — core's CI is their validation surface. `bridge/CORE_REF` must name a
commit that exists on GitHub, so push core before pushing here.

**Do not run `just prepare` while the patch is live.** It bakes core's own
queries into this repo's `.sqlx/`.

## Landing `next` on `main` (once core X.Y.Z is on crates.io)

```bash
# Make the override inert again in BOTH manifests (back under
# [workspace.metadata.unreleased-core-patch], marker removed), then:
just core-bump X.Y.Z             # every pin + CORE_REF, both lockfiles, local migrate, build, clippy
just schema-baseline             # record tests/fixtures/schema/release-baseline-X.Y.Z.sql
# write the CHANGELOG `## [X.Y.Z] - YYYY-MM-DD` entry
just verify
git push origin next             # the full Gates matrix runs on this exact commit
just release X.Y.Z               # opens the frozen promotion PR
just release X.Y.Z               # after its proof is green: merges and tags vX.Y.Z
```

`just release` refuses when the tree is dirty, when run off `next` (a clean
detached worktree at `origin/next` also qualifies), when a patch block is
live, when a pin or `CORE_REF` disagrees, when the schema ladder has no rung
for the version, when the changelog has no heading for it, when `HEAD` is not
`origin/next`, or when `main` is not an ancestor of it.

Afterwards, to resume development against the next unreleased core, make the
patch live again, `just core-pin`, and move the pins to the sibling's version.
