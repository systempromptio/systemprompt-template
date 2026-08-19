# Releasing

The process for shipping a new gateway release when a new `systemprompt` core
version lands on crates.io.

**This repository runs no hosted CI.** There are no GitHub Actions workflows:
it is a private repo and paying for runner minutes buys nothing a local
machine cannot do. Every gate is a local command, and every artifact is built
by hand. Nothing happens automatically when you push a branch or a tag — if
you did not run it, it did not run.

## Versioning policy

The fork tracks core in lockstep: core `X.Y.Z` on crates.io → workspace
`version = X.Y.Z` → git tag `vX.Y.Z` → Helm `appVersion: X.Y.Z` (the chart's
own `version:` gets a minor bump per release, handled by the sync script).

## Step A0 — adopting an *unpublished* core (the patched path)

Most core versions are adopted here before they are on crates.io: the sibling
`../systemprompt-core` checkout is bumped, this repo is patched onto it via
`[patch.crates-io]`, and the two are proven together *before* core publishes.
`just core-bump` deliberately refuses to run in this state — it is the
published-crates path — so this step is by hand and nothing reminds you.

**Bump the pins first, and bump all of them.** A version requirement that no
longer matches the patched crate does not error: cargo silently drops the
patch and resolves the old version from crates.io, so the build "works" while
proving nothing about the new core. The pins live in **two** manifests, because
`tests/` is a separate workspace with its own copy:

```bash
grep -rn '^systemprompt[a-z-]* = { version = "' --include=Cargo.toml . | grep -v target
sed -i 's/OLD/NEW/g' Cargo.toml tests/Cargo.toml     # or let the script do it
scripts/sync-release-version.sh NEW --check          # core-pin lines must be silent
```

`sync-release-version.sh` covers every core pin in both manifests plus a
residual sweep that fails on any core pin it does not itself move — so a pin
added to a new crate cannot sit stale. Its remaining `DRIFT:` lines on this
path are the *product* version (workspace version, Chart.yaml, deploy files);
those belong to Step A, not here. Do **not** bump them for a core that has not
shipped.

Then prove it, in this order — each step catches a class the previous one
cannot:

```bash
just build                                    # patch resolved? log must read the new version
just clippy
grep -n 'Breaking' ../systemprompt-core/CHANGELOG.md   # then grep this repo for each item
./target/debug/systemprompt infra db migrate  # new core migrations, against the local DB
./target/debug/systemprompt --version         # must print the new core version
just start && curl -s localhost:8080/health   # must reach {"status":"healthy"}, not "starting"
```

Confirm the build log names `systemprompt-* vNEW (/var/www/html/systemprompt-core/...)`.
A build that compiles registry crates instead is a dropped patch, not a pass.

Three things no gate catches on this path:

- **A tightened identifier validator is a runtime panic, not a compile error.**
  Core's `define_id!(…, validated, …)` types panic in `new()` on a value they
  used to accept, so a construction site that stops being legal still compiles
  and still passes clippy — it fails only when that code path executes. 0.29.0
  did exactly this to `ContextId` (now UUID-v4 only), and
  `hooks_track::build_request_context` had been passing `ContextId::new("")`,
  which would have panicked on every `/hooks/track` AI summary. Sweep for it
  whenever the core diff touches `crates/shared/identifiers`:
  `grep -rn '::new("' --include='*.rs' extensions/ src/` — and prefer
  `try_new` or `generate()` over a literal at any site that cannot prove the
  value's shape.

- **Migrations run silently and are not reversible.** Run them and then check
  the tables the core changelog describes actually exist, rather than trusting
  the success line.
- **A new core job is inert until this repo schedules it.** Core discovers jobs
  by inventory; whether one *runs* comes from `services/scheduler/config.yaml`.
  Boot warns `job is available in this build but has no scheduler.jobs entry`
  once per job and then carries on. Decide per job — scheduling it and
  deliberately leaving it off are both fine, silently missing it is not.

Only once core is published on crates.io do you comment the two
`[patch.crates-io]` blocks (root and `tests/`, in lockstep) and continue with
Step A.

## Step A — bump and validate locally

```bash
just core-bump X.Y.Z
```

This refuses to run with an active `[patch.crates-io]` override, then runs
`scripts/sync-release-version.sh X.Y.Z` (bumps the workspace version, the
`systemprompt` + `systemprompt-security` pins, Chart.yaml appVersion +
chart version + artifacthub annotation/changelog, and the exact-pin deploy
files: CasaOS compose, DigitalOcean compose + Packer default), runs
`cargo update -w`, migrations against the local DB, `just build`, and
`just clippy`.

Then exercise anything the core changelog touches, review the diff, and run
the full gate:

```bash
just verify
```

`verify` is the whole check in one command — `cargo fmt --check`, the offline
sqlx cache, the 23 source gates, clippy at `-D warnings`, and the unit,
integration, and admin-contract test suites. It is what a CI pipeline would
have run. Commit to main and push only once it is green.

## Step B — tag

```bash
just release X.Y.Z
```

Checks the tree is clean, HEAD == origin/main, every pin matches
(`sync-release-version.sh --check`), and `just verify` passes, then pushes the
`vX.Y.Z` tag. The tag is a marker: nothing consumes it.

## Step C — build and publish artifacts

By hand, from a clean checkout of the tag:

```bash
just build-all                    # release binary, MCP servers, web assets
just docker-build X.Y.Z           # container image, if one is being shipped
```

Push the image, attach binaries to a GitHub release, and package the Helm
chart only if that release is actually being distributed. Most releases of
this fork are deployed straight to the instance with `just deploy` and need
none of it.

## Rollback

1. Redeploy the previous good build (`just deploy` from the previous tag).
2. Mark any GitHub Release as pre-release or delete it.
3. Never reuse a tag — fix forward and cut the next patch version.
4. Chart: publish the previous chart again or a new patch chart pinning the
   good image via `image.tag`.

## Post-release checklist

- [ ] `just verify` green on the tagged commit
- [ ] the deployed instance serves the new version (`just server-status`)
- [ ] any published image or chart actually pushed — nothing does it for you
- [ ] update docs-internal/STATE.md release row
