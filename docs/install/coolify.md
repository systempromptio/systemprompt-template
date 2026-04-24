# Deploy the gateway to Coolify

Deploys the `systemprompt-gateway` server on [Coolify](https://coolify.io), a self-hostable Heroku-style platform. The service template provisions the gateway + Postgres from the GHCR image. For the cowork client CLI, see [../cowork/](../cowork/).

## Install from the Coolify community templates

1. Open your Coolify dashboard → **Services** → **New Service**.
2. Search for `systemprompt` in the community templates list, or paste the raw template URL:
   ```
   https://raw.githubusercontent.com/systempromptio/systemprompt-template/main/deploy/coolify/service-template.json
   ```
3. Set environment variables:
   - `POSTGRES_PASSWORD` — strong random
   - `ANTHROPIC_API_KEY` and/or `OPENAI_API_KEY` / `GEMINI_API_KEY`
4. Deploy.

## Manual (docker-compose)

If you prefer to paste the compose directly into Coolify's *Docker Compose* service type, use [`docker-compose.yml`](https://github.com/systempromptio/systemprompt-template/blob/main/docker-compose.yml) from the repo root, swapping `build: .` for `image: ghcr.io/systempromptio/systemprompt-template:latest`.

## Domain + TLS

Coolify terminates TLS. Point your domain at Coolify, set it on the gateway service, and it'll provision Let's Encrypt automatically.

Docs: https://systemprompt.io/documentation/?utm_source=coolify&utm_medium=install_doc
