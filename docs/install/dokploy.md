# Deploy the gateway on Dokploy

Deploys the `systemprompt-gateway` server on [Dokploy](https://dokploy.com), a self-hostable PaaS. The blueprint provisions the gateway + Postgres from the GHCR image.

Upstream template PR: [Dokploy/templates#1032](https://github.com/Dokploy/templates/pull/1032). Once merged, **systemprompt** appears in every Dokploy install's template picker and on [templates.dokploy.com](https://templates.dokploy.com) — until then use the manual import below.

## Install (base64 import)

1. Generate the import payload from [`deploy/dokploy/`](https://github.com/systempromptio/systemprompt-template/tree/main/deploy/dokploy):

   ```bash
   python3 - <<'EOF'
   import json, base64, urllib.request
   raw = "https://raw.githubusercontent.com/systempromptio/systemprompt-template/main/deploy/dokploy/"
   payload = {k: urllib.request.urlopen(raw + f).read().decode()
              for k, f in [("compose", "docker-compose.yml"), ("config", "template.toml")]}
   print(base64.b64encode(json.dumps(payload).encode()).decode())
   EOF
   ```

2. Dashboard → your project → **Create Service** → **Compose**, then open the service's **Advanced** section → **Import** → paste the base64 → **Import**. The compose file, environment (with a generated `POSTGRES_PASSWORD`), and a domain bound to port 8080 are populated for you.
3. In **Environment**, set at least one provider key **before the first deploy** — `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, or `GEMINI_API_KEY`. The container refuses to boot without one; blank values are safely ignored.
4. In **Domains**, enable **HTTPS** (Let's Encrypt) on the generated domain. This is required, not optional: the gateway validates `EXTERNAL_URL` and rejects plain `http://` origins other than localhost — with HTTPS off the container restart-loops with an "Invalid CORS origin" error.
5. Deploy. First boot runs migrations and the publish pipeline — allow up to 5 minutes before `/api/v1/health` reports healthy. Then open the domain root for the admin UI.

## Domain + TLS

The template maps your chosen domain to the gateway's port 8080 through Dokploy's Traefik proxy. The domain is injected as `EXTERNAL_URL` (always `https://`), so the gateway advertises the right public URL and CORS origin out of the box. `POSTGRES_PASSWORD` is auto-generated; if you set it manually, do so **before the first deploy** (Postgres initialises its volume with the first password it sees).

Docs: https://systemprompt.io/documentation/?utm_source=dokploy&utm_medium=install_doc
