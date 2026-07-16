# Deploy the gateway to Coolify

Deploys the `systemprompt-gateway` server on [Coolify](https://coolify.io), a self-hostable Heroku-style platform. The service template provisions the gateway + Postgres from the GHCR image. For the bridge client, see [../cowork/](../cowork/).

## Install (docker-compose service)

Paste the compose in directly:

1. Open your Coolify dashboard → **Services** → **New Service** → **Docker Compose**.
2. Paste the compose from [`deploy/coolify/service-template.json`](https://github.com/systempromptio/systemprompt-template/blob/main/deploy/coolify/service-template.json) (the `compose` field), or use [`docker-compose.yml`](https://github.com/systempromptio/systemprompt-template/blob/main/docker-compose.yml) from the repo root with `build: .` swapped for `image: ghcr.io/systempromptio/systemprompt-template:latest`.
3. Coolify pre-creates the env vars it finds in the compose — fill them in on the **Environment Variables** tab:
   - `POSTGRES_PASSWORD` — strong random. Set it **before the first deploy**: Postgres initialises its volume with whatever password is present on first boot, and changing it later fails auth until the volume is wiped.
   - `ANTHROPIC_API_KEY` and/or `OPENAI_API_KEY` / `GEMINI_API_KEY` — at least one; the container refuses to boot without a provider key.
4. Deploy. The gateway reports healthy on `/api/v1/health` about a minute after first boot (migrations run on start).
5. Coolify ignores the compose `ports:` mapping — it exposes services through its Traefik proxy. Set a domain on the `gateway` service (port 8080) to reach it; the health status in the dashboard works either way.

Verified end-to-end on Coolify 4.1.2 (2026-07-16).

## Domain + TLS

Coolify terminates TLS. Point your domain at Coolify, set it on the gateway service, and it'll provision Let's Encrypt automatically.

Docs: https://systemprompt.io/documentation/?utm_source=coolify&utm_medium=install_doc
