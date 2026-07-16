# Deploy target status

Per-platform status of every one-click deploy surface: where it stands, how to
test it via the platform's UI, and how to promote it to their community.
Snapshot 2026-07-16 (v0.21.0 release day). Detailed runbooks:
`docs-internal/testing/<platform>.md`; consolidated human actions:
`docs-internal/HUMAN-ACTIONS.md` (Sections E/F).

GitHub release notes now carry the full deploy-target matrix (templated in
`.github/workflows/release-gateway.yml`); the canonical channel table stays in
`docs/README.md`.

> **Gate:** the catalog templates pin `:0` / `:0.21.0`. Those tags exist only
> once the v0.21.0 multi-arch image build completes — verify with
> `docker manifest inspect ghcr.io/systempromptio/systemprompt-template:0.21.0`
> before testing anything below. `:latest` already serves 0.21.0 content, and
> the Helm chart (appVersion 0.21.0) is live on charts.systemprompt.io.

## 1. Railway — LIVE (published template)

- **Status:** template `systempromptio-the-self-owned-ai-control` published;
  CI re-verifies PUBLISHED on every release (smoke-tests `railway-template` job).
- **Test:** railway.com → template page → Deploy → set an Anthropic key →
  wait for healthy → `https://<generated>.up.railway.app/api/v1/health`.
- **Promote:** share URL is public; request template verification in the
  Railway dashboard (verified tier + kickback program); Discord #showcase.

## 2. Render — LIVE (blueprint)

- **Status:** `render.yaml` + working deploy button. Uses `:latest`, which now
  moves only on releases.
- **Test:** click the README deploy button, complete the wizard with a
  provider key, health-check the generated URL, delete the service.
- **Promote:** no community catalog; the deploy button and their community
  forum are the surface. Nothing to submit.

## 3. Coolify — VERIFIED, catalog PR in flight

- **Status:** verified e2e on Coolify 4.1.2. Upstream PRs awaiting review:
  coollabsio/coolify#10958 (template), coolify-docs#660, awesome-list #7.
- **Test:** Coolify → Services → New → Docker Compose → paste compose from
  `deploy/coolify/service-template.json` → env vars → deploy. Re-test on 0.21.0.
- **Promote:** on merge of #10958 the template ships with the next Coolify
  release; then Discord showcase + X post tagging @coolifyio (drafts agreed,
  Ed posts).

## 4. Dokploy — VERIFIED, awaiting submission

- **Status:** verified e2e 2026-07-16 on a fresh Dokploy install (2GB VM):
  base64 import → HTTPS on domain → deploy → health 200 → `/v1/messages` 200
  with audit rows. `deploy/dokploy/` now also ships `instructions.md`
  (HTTPS-required + provider-key notes). `generate-meta.js --check` passes
  with the blueprint in place. PR payload + description ready for Ed.
- **Test (re-run):** 2GB VPS → `curl -sSL https://dokploy.com/install.sh | sh`
  → project → Create Service → Template → Import (base64 of compose +
  template.toml) → set a real key → **enable HTTPS on the domain** (app
  refuses http CORS origins; restart-loops otherwise) → ~5 min →
  `/api/v1/health`. Full runbook: `docs-internal/testing/dokploy.md`.
- **Promote:** PR to `Dokploy/templates` (`blueprints/systemprompt/` + meta
  registration per their AGENTS.md). Each PR gets an automatic preview deploy.

## 5. Portainer — OWN FEED LIVE, community PRs pending

- **Status:** `deploy/portainer/templates.json` on main — the raw GitHub URL
  already works as a custom App Templates feed. No community listing yet.
- **Test:** local Portainer CE → Settings → App Templates → point at the raw
  templates.json URL → deploy the systemprompt template → health on `:8080`.
- **Promote:** PR to `Lissy93/portainer-templates` (community, fast, wide
  reach) and to `portainer/templates` v3 branch (official, slow/selective).

## 6. CapRover — ASSETS READY, not submitted

- **Status:** `deploy/caprover/systemprompt.yml` (captainVersion 4) drafted,
  YAML-valid, untested live.
- **Test:** CapRover on a cheap VPS → Apps → One-Click Apps → **Template**
  (bottom) → paste YAML → fill form → deploy →
  `http://<app>.<root>/api/v1/health` → enable HTTPS → update `EXTERNAL_URL`.
- **Promote:** PR to `caprover/one-click-apps` (`public/v4/apps/` + logo in
  `public/v4/logos/`); run `npm run validate_apps` in the fork first. Merged
  apps appear in every CapRover install.

## 7. CasaOS — TESTED, draft PR open (IceWhaleTech/CasaOS-AppStore#981)

- **Status:** `deploy/casaos/docker-compose.yml` with x-casaos metadata done
  (image pin auto-synced per release). `deploy/casaos/screenshot-1.png`
  captured from a live run and pushed to main (raw URL verified).
- **Test (done 2026-07-16):** exact manifest booted end-to-end against the
  pinned 0.21.0 GHCR image with a real Anthropic key — migrations ran, app
  healthy in ~20s, `/health` + `/v1/models` 200, restart on persisted
  volumes recovered in 10s. Optional remaining step: import into a real
  CasaOS UI in an Ubuntu VM (`curl -fsSL https://get.casaos.io | sudo bash`)
  to see the app tile.
- **Promote:** draft PR IceWhaleTech/CasaOS-AppStore#981 opened 2026-07-16
  (`Apps/SystemPrompt/`, v2 store protocol: reverse-domain id, en_US locale
  keys, version/update_at/release_notes, local icon.svg + screenshot;
  `./scripts/build_dist.sh` validated in Docker). Ed reviews, then marks
  ready; their review takes weeks. Low ICP fit — backlink/discovery value.

## 8. Zeabur — TEMPLATE DRAFTED, self-serve publish

- **Status:** `deploy/zeabur/template.yaml` drafted; their CLI validates the
  schema at publish (expect small fixes: `${PASSWORD}` helper, env expansion).
- **Test:** `npx zeabur auth login` → `npx zeabur template create -f
  deploy/zeabur/template.yaml` → deploy into a test project with a real key →
  health on the bound domain → delete project, keep template.
- **Promote:** publishing IS the listing (instant). Paste the share URL into
  `docs/install/zeabur.md` + README; request "featured" review once verified.

## 9. Northflank — VERIFIED E2E, share link + listing email pending (Ed)

- **Status:** verified e2e 2026-07-16 via their API on Ed's team: template
  `systemprompt` created, run converged (project + postgresql addon +
  gateway + secret group + volume all green), health 200 and admin UI served
  on the generated `code.run` domain. Live-run fixes now in
  `deploy/northflank/template.json`: `POSTGRES_URI_ADMIN` (the non-admin
  addon user cannot CREATE SCHEMA), `successThreshold` required on
  readinessProbe, no `storageClass` (feature-flagged), `nf-compute-20`
  plans (free-tier allowance), volume >= 6144MB (nvme minimum), no em
  dashes in `description` (charset-validated). Findings + listing email
  draft in `docs-internal/testing/northflank.md`.
- **Test (re-run):** `POST /v1/templates/systemprompt/runs` with
  `NORTHFLANK_TOKEN` from `.env.secrets`, then health on
  `https://web--gateway--<ns>.code.run/api/v1/health`.
- **Promote:** the saved template yields a shareable one-click deploy link
  immediately; northflank.com/stacks listing is curated — email their
  partnerships/support with the verified link (no cost, no licence gate).

## 10. DigitalOcean Marketplace — BUILD READY, vendor application is the long pole

- **Status:** complete Packer 1-Click image under `deploy/digitalocean/`
  (bundled Postgres, first-login key setup, systemd-gated stack, pins synced
  per release). Nothing submitted.
- **Test:** `packer build` into your DO account (runbook
  `docs-internal/testing/digitalocean.md`) → droplet from the private
  snapshot via DO UI → SSH → MOTD/setup flow → health + reboot persistence →
  official `img_check.sh`.
- **Promote:** **file the vendor application now**
  (marketplace.digitalocean.com/vendors — free, 2–4 week review, runs in
  parallel). On approval, submit snapshot ID + listing copy + 512px icon
  (`storage/files/images/icon-512.png`) via the Vendor Portal.

## 11. Elestio — HAND-OFF READY, outreach-shaped

- **Status:** `deploy/elestio/` compose + README prepared; email draft in
  `docs-internal/testing/elestio.md`.
- **Test:** not possible — Elestio builds and operates stacks internally; no
  self-serve path.
- **Promote:** send the drafted partnership email + file their "request new
  software" form. Their timeline (weeks–months). Public install doc only
  after they list it.

## 12. Umbrel — DEFERRED (product blocker)

Umbrel forbids install-time env prompts and requires apps to open into a
working setup flow without SSH; the container refuses to boot without a
provider key. Needs a "no-key setup mode" (boot bare, collect the key in the
admin UI) before a `getumbrel/umbrel-apps` PR is viable. That feature would
also improve the CasaOS/Portainer experience.

## 13. Cloudron — DEFERRED (highest effort)

Not compose-based: full custom package (CloudronManifest.json, their
filesystem/addon/lifecycle conventions, `cloudron build`) plus a conservative
team review, for modest audience overlap. Revisit on demand.

---

## Suggested testing order

1. Portainer (zero cost, local Docker)
2. Dokploy + CapRover (one throwaway VPS each)
3. Zeabur + Northflank (browser only)
4. CasaOS (VM + capture the screenshot)
5. DigitalOcean Packer build

File the DO vendor application and send the Elestio email **first** — both
are wait-driven and run in parallel with everything else. All submissions and
GitHub/marketplace actions are executed by Ed.
