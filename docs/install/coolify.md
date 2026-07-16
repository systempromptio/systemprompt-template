# Deploy the gateway to Coolify

Deploys the `systemprompt-gateway` server on [Coolify](https://coolify.io), a self-hostable Heroku-style platform. The service template provisions the gateway + Postgres from the GHCR image. For the bridge client, see [../cowork/](../cowork/).

## Install (docker-compose service)

Paste the compose in directly:

1. Open your Coolify dashboard → **Services** → **New Service** → **Docker Compose**.
2. Paste the compose from [`deploy/coolify/service-template.json`](https://github.com/systempromptio/systemprompt-template/blob/main/deploy/coolify/service-template.json) (the `compose` field), or use [`docker-compose.yml`](https://github.com/systempromptio/systemprompt-template/blob/main/docker-compose.yml) from the repo root with `build: .` swapped for `image: ghcr.io/systempromptio/systemprompt-template:latest`.
3. Set environment variables:
   - `POSTGRES_PASSWORD` — strong random
   - `ANTHROPIC_API_KEY` and/or `OPENAI_API_KEY` / `GEMINI_API_KEY`
4. Deploy. The gateway reports healthy on `/api/v1/health` about a minute after first boot (migrations run on start).

Once the upstream template merges, **Services → New Service → search "systemprompt"** replaces steps 1-2.

## Domain + TLS

Coolify terminates TLS. Point your domain at Coolify, set it on the gateway service, and it'll provision Let's Encrypt automatically.

Docs: https://systemprompt.io/documentation/?utm_source=coolify&utm_medium=install_doc
