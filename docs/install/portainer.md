# Deploy the gateway on Portainer

Deploys the `systemprompt-gateway` server as a Portainer stack (app template). The template provisions the gateway + Postgres from the GHCR image.

## Install (app template feed)

1. Portainer → **Settings** → **App Templates** → set the URL to:
   `https://raw.githubusercontent.com/systempromptio/systemprompt-template/main/deploy/portainer/templates.json`
   (or merge our entry into your existing custom feed).
2. **App Templates** → **systemprompt** → fill in the form:
   - `POSTGRES_PASSWORD`: strong random, **before first deploy** (the database volume is initialised with it).
   - `ANTHROPIC_API_KEY` and/or `OPENAI_API_KEY` / `GEMINI_API_KEY`: at least one; the container refuses to boot without a provider key.
   - `EXTERNAL_URL`: optional; the public URL you'll serve the gateway on (Portainer doesn't inject one automatically).
3. Deploy the stack. First boot runs migrations and the publish pipeline: allow several minutes before `/api/v1/health` on `http://<host>:8080` returns 200.

Alternatively skip templates entirely: **Stacks** → **Add stack** → **Repository**, URL `https://github.com/systempromptio/systemprompt-template`, compose path `deploy/compose/one-click.docker-compose.yml`.

## TLS

Portainer doesn't terminate TLS for stacks; put your usual reverse proxy in front of port 8080 and set `EXTERNAL_URL` to match.

Docs: https://systemprompt.io/documentation/?utm_source=portainer&utm_medium=install_doc
