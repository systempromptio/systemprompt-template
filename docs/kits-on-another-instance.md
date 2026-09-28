# Composing kits on another instance

A kit is published once, by its repository's CI, as a signed services bundle
on GHCR. Any systemprompt instance built from this repository can compose it;
nothing in a kit is instance-specific. This is what the operator of a second
deployment does to follow the same kits as the first. The model is
`/documentation/services-sync`; the kit contract is
`deploy/kit/README-INTEGRATION.md`; the registry of kits is
`deploy/kit/known-kits.json`.

## 1. A pull token, for private packages only

A kit whose GHCR package is public (`pull: public` in `known-kits.json`) needs
no credential. A private one is pulled with one secret:

| secret | value |
|---|---|
| `ghcr_pull_token` | `<github-user>:<PAT>` — a GitHub personal access token (classic) with the single scope `read:packages`, on an account that can read the owning organisation's packages |

Put it in the profile's `secrets.json` (or the equivalent secret store the
deployment uses; `secrets.source` in the profile says which).

## 2. The sources block

Add to the instance's profile (`.systemprompt/profiles/<name>/profile.yaml`),
one entry per kit:

```yaml
services:
  sources:
    - name: <kit>
      oci:
        reference: ghcr.io/<org>/<kit-repo>:stable
        auth_secret: ghcr_pull_token          # private packages only
        verify:
          ed25519_public_keys: ["<the kit's public key>"]
  cache_dir: /app/services-cache        # any writable directory
  on_fetch_failure: use_last_good       # use_bundled on the very first boot
```

The public key is the one recorded for the kit in `deploy/kit/known-kits.json`;
a bundle signed by any other key is refused. `just services-pin <kit> stable
<profile>` writes exactly this entry from that registry (and omits
`auth_secret` for a public package). The shape is validated by
`docs/profile.schema.json`.

## 3. Entitlement stays in this repository

Who reaches a kit's marketplace is declared in
`services/access-control/rules.yaml` as `marketplace/<id>` with
`owner: bundle:<kit>`, and the groups and projects it names are declared in
`services/web/config/groups.yaml`. An instance deployed from this repository
carries both, so every instance composing the kit grants it to the same
audience; a kit release can never widen it.

## 4. Import after every kit release

A kit release moves its `stable` tag. The instance serves it after one
**Import sources** on `/admin/sync`, or:

```bash
curl -fsS -X POST -H "Authorization: Bearer <admin PAT>" \
  https://<instance>/api/v1/admin/services/refresh
# {"changed":true,"reconciled":true,"restart_recommended":false,...}
```

Nothing restarts. The kit CI can make this call itself for **one** instance
(its `SYSTEMPROMPT_API_URL` / `SYSTEMPROMPT_ADMIN_TOKEN` secrets); every other
instance imports on its own schedule. `GET /api/v1/admin/services/status`
shows the active digest per source.

## 5. Rollback

`just services-pin <kit> sha256:<previous digest> <profile>` and Import; the
cache keeps the last two trees per source. The digests are in each kit's
release run summary and on the GHCR package page.
