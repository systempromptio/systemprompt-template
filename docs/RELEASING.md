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
review the diff, commit to main, push. This is the human gate.

## Step B — release

```bash
just release X.Y.Z
```

Checks the tree is clean, HEAD == origin/main, and every pin matches
(`sync-release-version.sh --check`), then pushes the `vX.Y.Z` tag. From here
everything is automatic:

| Workflow | Trigger | Produces |
|---|---|---|
| `docker.yml` | tag push | multi-arch (amd64+arm64) image, tags `X.Y.Z`/`X.Y`/`X`/`latest`, cosign-signed |
| `release-gateway.yml` | tag push | binary tarballs + SHA256SUMS + cosign sig on a GH Release; Homebrew formula bump |
| `helm.yml` | release published | chart packaged and pushed to charts.systemprompt.io |
| `smoke-tests.yml` | release published | install-channel smokes + `release-tags` (all tags one digest, both arches, signature verifies) + `helm-release` (chart serves the new appVersion) |
| `ghcr-prune.yml` | after Docker succeeds on a tag + weekly | retention (below) |

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

`ghcr-prune.yml`: keep the 5 newest release versions; delete `sha-*` tags and
untagged manifests older than 4 weeks. Alias tags always point at kept
digests. Dry-run available via workflow dispatch.

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
- [ ] update docs-internal/STATE.md release row
