# Deploy the gateway on CasaOS

Deploys the `systemprompt-gateway` server on [CasaOS](https://casaos.zimaspace.com) (gateway + bundled Postgres).

## Install

Once merged into the CasaOS App Store, install **systemprompt** from the store. Until then:

1. CasaOS dashboard → **App Store** → **Install a customized app** → **Import** → paste [`deploy/casaos/docker-compose.yml`](https://github.com/systempromptio/systemprompt-template/blob/main/deploy/casaos/docker-compose.yml).
2. Fill in the environment variables:
   - `ANTHROPIC_API_KEY` and/or `OPENAI_API_KEY` / `GEMINI_API_KEY`: at least one; the app will not start without a provider key.
   - `POSTGRES_PASSWORD`: set once, before first start.
3. Install. First start runs migrations and the publish pipeline: allow several minutes, then open the app (port 8080).

## Access beyond the LAN

CasaOS serves apps on the local network. If you expose the gateway publicly (reverse proxy, tunnel), set `EXTERNAL_URL` to that URL in the app settings and restart so API URL + CORS match.

Docs: https://systemprompt.io/documentation/?utm_source=casaos&utm_medium=install_doc
