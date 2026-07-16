# Deploy the gateway on Zeabur

Deploys the `systemprompt-gateway` server on [Zeabur](https://zeabur.com) (gateway + Postgres from the GHCR image).

## Install

Once the template is published, deploy from its Zeabur template page (link will land here). Manual path:

1. Create a project → **Deploy New Service** → **Docker Image** → `ghcr.io/systempromptio/systemprompt-template:0`, plus a `postgres:18-alpine` service (volume at `/var/lib/postgresql`).
2. On the gateway service set:
   - `DATABASE_URL` — `postgres://systemprompt:<password>@postgres.zeabur.internal:5432/systemprompt`
   - `ANTHROPIC_API_KEY` and/or `OPENAI_API_KEY` / `GEMINI_API_KEY` — at least one.
   - `EXTERNAL_URL` — the generated `https://<name>.zeabur.app` domain (the template wires this automatically).
3. Bind a domain to port 8080. First boot runs migrations and the publish pipeline — allow several minutes before `/api/v1/health` returns 200.

Docs: https://systemprompt.io/documentation/?utm_source=zeabur&utm_medium=install_doc
