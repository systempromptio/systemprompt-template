# Deploy the gateway to Render

One-click deploy of the `systemprompt-gateway` server on [Render](https://render.com) via Blueprint. For the bridge client, see [../cowork/](../cowork/).

## Deploy

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/systempromptio/systemprompt-template)

**In the Blueprint dialog, set "Blueprint Path" to `deploy/render/render.yaml`.** Render defaults to `render.yaml` at the repo root, which doesn't exist here, and the deploy button URL can't prefill the field. Without this step you'll get "Blueprint file render.yaml not found".

Render reads [`deploy/render/render.yaml`](https://github.com/systempromptio/systemprompt-template/blob/main/deploy/render/render.yaml) and provisions:
- A `systemprompt-gateway` web service from the GHCR image
- A `systemprompt-postgres` database (Render Postgres, free plan)
- `DATABASE_URL` wired in automatically

## Required env vars

In the Blueprint dialog, set at least one (they're declared with `sync: false` so Render prompts you):

- `ANTHROPIC_API_KEY`
- `OPENAI_API_KEY`
- `GEMINI_API_KEY`

## Scaling

Defaults to the `free` database plan and a single web instance. Edit `render.yaml` (fork → modify → reconnect Blueprint) to bump plans or add replicas.

Docs: https://systemprompt.io/documentation/?utm_source=render&utm_medium=install_doc
