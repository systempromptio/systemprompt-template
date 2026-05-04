# Cleanup: deploy machinery to migrate out of `systemprompt-template`

This repo is the **public fork-and-compile target** for systemprompt.io. It should contain the application skeleton, demos, install docs, and end-user-facing release surface — nothing more. CI / packaging / distribution machinery lives in the private `systemprompt-deploy` repo.

`systemprompt-template` was originally seeded from `systemprompt-deploy` (commit `c730c8e`). Several files inherited from that seeding are deploy-style infrastructure that does not belong in a public template. This document lists what should migrate back, what should stay, and how to do it.

For full context on the three-repo architecture, see [`systemprompt-core/documentation/cowork/`](../systemprompt-core/documentation/cowork/README.md).

## Migrate to `systemprompt-deploy`

### Definitely deploy-only (move without debate)

| Path | Why |
|---|---|
| `docs-internal/` | Distribution strategy, registry strategy, manual test procedures for apt/rpm/homebrew/scoop/nix/winget/helm/docker-hub/binary/coolify/railway/render. Forkers do not maintain these channels. Files: `HUMAN-ACTIONS.md`, `STATE.md`, `distribution-implementation-plan.md`, `distribution-strategy.md`, `docker-registry-strategy.md`, `manual-cowork-test.md`, `testing/`. |
| `.github/workflows/release.yml` | This workflow downloads cowork artifacts from `systemprompt-core` releases and re-bundles them. The template should not be running release pipelines. **Move into `systemprompt-deploy`** — it already cross-uploads gateway assets to template via `RELEASE_UPLOAD_TOKEN`; have it do the same for cowork. After the move, template becomes a passive Release surface for both products and runs zero workflows. The `cowork-v*` tag continues to live on template (that's where users discover it) but the workflow listening for that tag lives in deploy. |

### Strongly suggested to move (deploy-tuned, not user-friendly)

| Path | Why | Caveat |
|---|---|---|
| `Dockerfile` | Multi-stage build with sqlx offline mode, prod-tuned. Forkers running locally usually want a simpler dev compose. | Provide a slim `Dockerfile.dev` in template if dev-mode container builds remain a use case. |
| `docker-compose.yml` | Wired for prod-shape Postgres + app, not local hacking. | Same — keep a minimal dev-compose if useful. |
| `Cross.toml` | Exists for cross-compilation in CI. Forkers don't cross-compile. | n/a |
| `flake.nix`, `flake.lock` | Nix dev-shell — useful, but currently mirrors deploy's. | Optional; keep if it actually makes fork onboarding nicer. |

### Probably move (PaaS / installer config)

| Path | Why | Caveat |
|---|---|---|
| `deploy/coolify/`, `deploy/railway/`, `deploy/render/` | PaaS deploy descriptors. Could legitimately help a forker self-host, but the values reference systemprompt.io's demo-cloud configuration, not a generic fork. | If kept, sanitize values; if moved, replace with a `docs/deploy-paas.md` pointing at `systemprompt-deploy/deploy/`. |
| `scripts/install.sh`, `scripts/install-gateway.sh`, `scripts/install-cowork.sh`, `scripts/cowork-headless-login.sh` | Public installer scripts. Belong with the binaries they install. Candidates: live next to cowork (`systemprompt-core/bin/cowork/scripts/`) or in deploy alongside packaging. | Resolve the open question below first (where do public `curl \| sh` scripts live?). |

## Keep in template

These are legitimately user-facing and belong to the template's purpose:

- `Cargo.toml`, `Cargo.lock` — workspace declaration. Note: forkers must strip the `[patch.crates-io]` block (lines 244+).
- `src/`, `extensions/`, `services/`, `migrations/`, `web/`, `content/`, `storage/`, `data/` — application skeleton.
- `demo/` — 43 evaluation scenarios. Users clone and run these.
- `docs/install/*` — public install docs (binary, GHCR, Helm, Homebrew, Nix, Railway, Render, Coolify).
- `docs/cowork/*` — cowork client install + device-auth docs (macOS, Windows, Scoop, Windows demo).
- `README.md`, `LICENSE`, `AGENTS.md`, `CLAUDE.md`.
- `justfile` — local-build recipes.
- `deny.toml` — cargo-deny config; lightweight, fine to keep.
- `.systemprompt/` — gitignored runtime state; leave as-is.

## Migration steps

For each path you decide to move:

1. **Extract with history** (recommended over `cp` so blame survives):
   ```bash
   cd /var/www/html/systemprompt-template
   git format-patch HEAD~N -- <path> -o /tmp/patches/<path>
   cd /var/www/html/systemprompt-deploy
   git am /tmp/patches/<path>/*.patch
   ```
   For complex extractions, use `git filter-repo --path <path>` against a clone.

2. **Delete from template** in a separate commit so the move is reviewable:
   ```bash
   cd /var/www/html/systemprompt-template
   git rm -r <path>
   git commit -m "cleanup: move <path> to systemprompt-deploy"
   ```

3. **Update template's `README.md`** to point users at the deploy repo (or wherever the public installer ends up living) for any moved scripts.

4. **Update deploy's `README.md`** to document the newly-owned paths.

5. **Verify**:
   - Template still compiles: `just build` or `cargo build --workspace`.
   - Any retained workflow in template (if you keep one) still runs end-to-end.
   - Cross-repo references resolve: deploy's `release.yml` still uploads to template's GH Releases; template's docs still link to live install URLs.

## Open questions (resolve before migrating)

These shape *where* moved files end up:

1. **Public installer scripts**: should `install-cowork.sh` and `install-gateway.sh` live next to the binary (`systemprompt-core/bin/cowork/scripts/` and `systemprompt-deploy/scripts/`), or in a single fourth repo (`systemprompt-installers`) so `get.systemprompt.io` has one canonical source?
2. **Gateway upload target**: deploy currently uploads gateway assets to template's GH Releases (via `RELEASE_UPLOAD_TOKEN`). Should template's Releases page remain the canonical user download surface, or should it become deploy's, with template just linking out? Affects every install doc URL.
3. **PaaS configs (`deploy/coolify`, `deploy/railway`, `deploy/render`)**: are these intended as starting points for forkers (keep), or as the live demo-cloud's deployment config (move to deploy)?

Until these are resolved, hold off on moving anything in the "Probably move" section.
