# Releasing

The canonical process for shipping a gateway release when a new
`systemprompt` core version lands on crates.io. Manual at the front (a core
bump is never consumed blind), frozen in the middle (the commit that ships is
the commit the gates proved), automatic after the tag. The branch contract it
relies on is in [BRANCHING.md](BRANCHING.md).

## Versioning policy

The template tracks core in lockstep: core `X.Y.Z` on crates.io → workspace
`version = X.Y.Z` → git tag `vX.Y.Z` → image tags `X.Y.Z` / `X.Y` / `X` /
`latest` → Helm `appVersion: X.Y.Z` (the chart's own `version:` gets a minor
bump per release, handled by the sync script). `sync-release-version.sh
--check` refuses a release whose `MAJOR.MINOR` differs from the core pin's; a
template-only patch (`X.Y.1` on core `X.Y.0`) is the one sanctioned
divergence.

## Step A — adopt the core release on `next`

```bash
just core-bump X.Y.Z
```

This refuses to run with a live `[patch.crates-io]` override, then runs
`scripts/sync-release-version.sh X.Y.Z` (workspace version, Chart.yaml
appVersion + chart version + artifacthub annotation/changelog, the exact-pin
deploy files — CasaOS compose, DigitalOcean compose + Packer default — and
the version literals in `docs/install/*.md` / `deploy/*/*.md`) and
`scripts/sync-core-version.sh X.Y.Z` (every core crate pin in both
workspaces, `bridge/CORE_REF = vX.Y.Z`), refreshes **both** lockfiles
(`cargo update -w`, root and `tests/`), migrates the **local** database with
the new binary (`--profile local`, never the active session's profile; a
failed migrate stops the bump), then `just build` and `just clippy`.

Then, before anything is pushed:

1. Read core's CHANGELOG for tightened identifier validators and new
   `NOT NULL` columns — runtime failures no compile catches.
2. `just prepare` if queries changed, then `just schema-baseline` to record
   the ladder rung `tests/fixtures/schema/release-baseline-X.Y.Z.sql`
   (`scripts/check-schema-baseline.sh` and `just release` refuse without it).
3. Write the `CHANGELOG.md` entry: `## [X.Y.Z] - YYYY-MM-DD`, bullets under
   `Breaking`, `Added`, `Changed`, `Fixed`, `Removed`; every breaking bullet
   leads with `**Breaking:**`, names the symbol, and ends `Migrate by …`.
   `sync-release-version.sh` never touches it — only a human knows what is
   breaking for a consumer.
4. `just verify` (the static, lint and test tiers gates.yml runs) and, with a
   stack up, `just e2e-gate`.
5. Commit to `next` and push. `gates.yml` runs the full matrix on that exact
   commit; its `Gates passed` job is the proof the release consumes.

## Step B — `just release X.Y.Z` (twice)

`main` is release-only and protected: pull request only, no bypass. A PR
headed at `next` would merge whatever `next` points at when it is merged, so
the release freezes the proven commit instead.

**First run** (from `next`, or a clean detached worktree at `origin/next`):
checks the tree is clean, no patch is live, every pin agrees
(`sync-release-version.sh X.Y.Z --check`, `sync-core-version.sh --check`,
`check-core-ref.sh`), the schema ladder has the rung, the changelog has the
heading, `HEAD == origin/next`, `main` is an ancestor of it, and the latest
`gates.yml` **push** run on that exact SHA succeeded with a successful
`Gates passed` job (`scripts/check-gates-green.sh`). It then pushes the
frozen ref `promote/X.Y.Z/<main-sha>/<candidate-sha>` and opens the PR onto
`main`. The PR's Gates run does not repeat the matrix: its `Verify frozen
promotion` job re-reads the push proof and checks that `main` and `next`
have not moved and that GitHub's merge commit has exactly the candidate's
tree.

**Second run**, once that PR run is green
(`scripts/check-promotion-green.sh`): re-checks the PR head/base, that
`main` has not moved, that `refs/pull/<n>/merge` has the proven tree and the
push proof still stands, merges with `--match-head-commit`, verifies the
merge's parents and tree, and pushes the `vX.Y.Z` tag at the merge commit. If
the tag push fails, re-running resumes at the tag.

Why the tag is pushed by the command and not by a workflow: everything
downstream runs on a `v*` tag push, and a tag pushed with a workflow's
`GITHUB_TOKEN` never starts another workflow. `check-release-tag.sh` (a lint
gate) keeps the invariant that every CHANGELOG version older than the
workspace version carries its tag, because the versioned image only exists if
the tag did.

The self-tests for all of this run in the `static` tier:
`tests/scripts/release-proof.sh`, `release-merge.sh`, `release-helper.sh`
(mocked `git`/`gh`; no network).

## Step C — automatic, from the tag

| Workflow | Trigger | Produces |
|---|---|---|
| `release-gateway.yml` | tag push | `resolve` re-verifies the promotion merge (`check-release-merge.sh`) and the pins (`validate-release.sh`); binary tarballs + `SHA256SUMS.gateway` + cosign sig on a GH Release; Homebrew formula bump |
| `docker.yml` | called by `release-gateway.yml` | multi-arch image (amd64+arm64): per-arch `smoke-image.sh` on each candidate digest, manifest `:sha-<7>` cosign-signed, **`upgrade-boot`** over every schema rung, then **`promote-tags`** `:X.Y.Z`/`:X.Y`/`:X`/`:latest` at that proven digest |
| `helm.yml` | called after image publication | required kind install/test, then chart publication |
| `smoke-tests.yml` | called by `release-gateway.yml`, after the image and chart | install-channel smokes + `release-tags` (all tags one digest, both arches, signature verifies) + `helm-release` (chart serves the new appVersion) |
| `ghcr-prune.yml` | after Docker succeeds on a tag + weekly | retention (below) |

`upgrade-boot` restores each `tests/fixtures/schema/release-baseline-*.sql`
into Postgres 18, seeds 2000 rows into every hot table
(`tests/integration/schema-upgrade/src/seed_hot_tables.sql`), boots the
candidate image over it with a fresh `/app/storage/data` volume, requires a
finished-boot `/health` body (`{"status":"healthy"}` — the early listener
answers 200 with `"starting"`) or `/readyz`, and checks the rows survived. An
empty ladder fails the job. Only after it passes do the release aliases move,
so an alias never names an image that failed to boot over an old release's
database; a failed run leaves only its `:sha-*`.

Image and smoke tests are `workflow_call` jobs inside the `release-gateway.yml`
run, not separate event-triggered workflows. They used to listen for
`release: published`, which never fired: `gh release create` runs as the
default `GITHUB_TOKEN`, and events raised by that token do not start workflow
runs. v0.23.0 was tagged, the release published, and no image was built at all
until `docker.yml` was dispatched by hand. If you split them back out, use a
PAT, not `github.token`. A manual `release-gateway.yml` dispatch (rebuilding
an existing tag) skips the merge proof.

**A release is done when `smoke-tests` is fully green.** Until then, don't
advertise it or update marketplace listings.

Coverage (`coverage.yml`) measures `main`, nightly on `next`, and on demand.
It is a measurement, not a release check.

## Image tag semantics

- `:latest` — newest **release** (re-pointed only by `promote-tags`, after the proofs).
- `:X` / `:X.Y` — float within major/minor; what catalog templates pin (`:0`).
- `:X.Y.Z` — immutable release pin; what Helm resolves via appVersion.
- `:edge` + `:sha-<sha>` — every main push (and `:sha-<sha>` for every release candidate); development only, never advertised.

Consumers pick up releases on their next pull: `helm repo update && helm
upgrade`, `docker compose pull && up -d`, or a platform redeploy
(Render/Railway re-resolve `:latest` on redeploy; registry pushes alone do
not force a redeploy — that's platform behaviour). The DigitalOcean droplet
image is pinned at Packer build time and needs a rebuild + marketplace
update per release (see docs-internal/testing/digitalocean.md).

## Retention

`ghcr-prune.yml` uses the publishing repository's short-lived `GITHUB_TOKEN`
with package admin access. Keep the 5 newest release versions; remove stale
sha tags and untagged manifests after four weeks. Retention action v3.1.0
protects children of retained multi-architecture manifests. Use workflow
`dry_run=true` to review candidates before applying a changed retention policy.

Nuance: a version still carrying an alias tag (`X.Y` or `X`) is not matched
by the three-part filter and therefore never pruned — by design, since
deleting it would break the alias. Only versions left with a bare `X.Y.Z`
tag (aliases moved on) enter the keep-5 window. Fully dead lines (e.g. the
pre-lockstep 0.4/0.5 era) are removed by hand:
`DELETE /orgs/systempromptio/packages/container/systemprompt-template/versions/<id>`
with the `GHCR_PRUNE_TOKEN`.

## Rollback

1. Re-point `latest` to the previous good release:
   `crane tag ghcr.io/systempromptio/systemprompt-template:X.Y.(Z-1) latest`
   (same for the `:X` and `:X.Y` aliases if the bad release moved them).
2. Mark the GitHub Release as pre-release or delete it.
3. Never reuse a tag — fix forward and cut the next patch version.
4. Chart: publish the previous chart again or a new patch chart pinning the
   good image via `image.tag`.

## Post-release checklist

- [ ] smoke-tests green (including `release-tags` + `helm-release`)
- [ ] one catalog deploy pulls the new version (e.g. `deploy/compose/one-click.docker-compose.yml`, which floats on `:0`)
- [ ] `ghcr-prune` ran clean; expected old versions removed
- [ ] rebuild + resubmit the DigitalOcean marketplace image (when listed)
- [ ] release notes deploy matrix matches [docs/README.md](README.md) channel table (the template lives in `.github/workflows/release-gateway.yml`)
- [ ] update docs-internal/STATE.md release row

## Release validation notes

Builds use the committed lockfiles and SQLx offline caches; migrations and dependency updates are explicit setup/maintenance steps. The in-repository proc-macro-error2 compatibility patch is documented in `vendor/README.md`.

Release dispatch resolves its tag to a commit on main, validates version pins, and uses that commit for archives, containers and deployment tests. Candidate images must boot per architecture and over every schema rung before receiving release aliases. Post-publication smoke tests check both architectures, fresh setup, restart and upgrade from 0.42.1 with retained users. Helm is installed against disposable Postgres before chart publication; Homebrew publication completes before install-channel smoke tests.

If cleanup cannot read the package, verify the repository's Actions access in
the GHCR package settings. Deletion requires the Admin role; never suppress
an authorization failure or replace it with an unconditional successful step.

Existing container profiles with pre-0.44 `providers` or `gateway` sections are migrated before CLI startup. The original profile is backed up as `profile.pre-0.47.yaml`; customized provider catalogs and routes are retained under `legacy-services/` in the same persistent profile volume and reapplied to the services tree at each boot. Read-only externally managed profiles must be migrated before mounting.
