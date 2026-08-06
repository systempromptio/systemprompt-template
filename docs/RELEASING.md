# Releasing

The canonical process for shipping a new gateway release when a new
`systemprompt` core version lands on crates.io. Deliberately manual at the
front (a core bump is never consumed blind), fully automatic after the tag.

## Versioning policy

The template tracks core in lockstep: core `X.Y.Z` on crates.io → workspace
`version = X.Y.Z` → git tag `vX.Y.Z` → image tags `X.Y.Z` / `X.Y` / `X` /
`latest` → Helm `appVersion: X.Y.Z` (the chart's own `version:` gets a minor
bump per release, handled by the sync script).

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

Then: run the test suite, exercise anything the core changelog touches,
**write the `CHANGELOG.md` entry for this version**, review the diff, commit
to main, push. This is the human gate.

`sync-release-version.sh` deliberately does not touch `CHANGELOG.md`: only a
human knows which of the release's changes are breaking for a consumer. Head
the entry `## [X.Y.Z] - YYYY-MM-DD` and group bullets under `Breaking`,
`Added`, `Changed`, `Fixed`, `Removed`. Every breaking bullet leads with
`**Breaking:**`, names the affected symbol, and ends with `Migrate by …`.

## Step B — release

```bash
just release X.Y.Z
```

Checks the tree is clean, HEAD == origin/main, and every pin matches
(`sync-release-version.sh --check`), then pushes the `vX.Y.Z` tag. From here
everything is automatic:

| Workflow | Trigger | Produces |
|---|---|---|
| `release-gateway.yml` | tag push | binary tarballs + SHA256SUMS + cosign sig on a GH Release; Homebrew formula bump |
| `docker.yml` | called by `release-gateway.yml` | multi-arch (amd64+arm64) image, tags `X.Y.Z`/`X.Y`/`X`/`latest`, cosign-signed |
| `smoke-tests.yml` | called by `release-gateway.yml`, after the image | install-channel smokes + `release-tags` (all tags one digest, both arches, signature verifies) + `helm-release` (chart serves the new appVersion) |
| `helm.yml` | push to main touching `helm/**` — every release commit bumps Chart.yaml | chart packaged and pushed to charts.systemprompt.io |
| `ghcr-prune.yml` | after Docker succeeds on a tag + weekly | retention (below) |

Image and smoke tests are `workflow_call` jobs inside the `release-gateway.yml`
run, not separate event-triggered workflows. They used to listen for
`release: published`, which never fired: `gh release create` runs as the
default `GITHUB_TOKEN`, and events raised by that token do not start workflow
runs. v0.23.0 was tagged, the release published, and no image was built at all
until `docker.yml` was dispatched by hand. If you split them back out, use a
PAT, not `github.token`.

**A release is done when `smoke-tests` is fully green.** Until then, don't
advertise it or update marketplace listings.

## Image tag semantics

- `:latest` — newest **release** (re-pointed only by `v*` tags).
- `:X` / `:X.Y` — float within major/minor; what catalog templates pin (`:0`).
- `:X.Y.Z` — immutable release pin; what Helm resolves via appVersion.
- `:edge` + `:sha-<sha>` — every main push; development only, never advertised.

Consumers pick up releases on their next pull: `helm repo update && helm
upgrade`, `docker compose pull && up -d`, or a platform redeploy
(Render/Railway re-resolve `:latest` on redeploy; registry pushes alone do
not force a redeploy — that's platform behaviour). The DigitalOcean droplet
image is pinned at Packer build time and needs a rebuild + marketplace
update per release (see docs-internal/testing/digitalocean.md).

## Retention

`ghcr-prune.yml` (needs the `GHCR_PRUNE_TOKEN` secret — classic PAT with read:packages + delete:packages; the Actions token cannot delete org-owned packages): keep the 5 newest release versions; delete `sha-*` tags and
untagged manifests older than 4 weeks. Alias tags always point at kept
digests. Dry-run available via workflow dispatch.

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
- [ ] release notes deploy matrix matches [docs/README.md](README.md) channel table (templates live in `.github/workflows/release-gateway.yml` and `release.yml`)
- [ ] update docs-internal/STATE.md release row
