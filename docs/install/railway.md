# Deploy the gateway to Railway

One-click deploy of the `systemprompt-gateway` server on [Railway](https://railway.com). For the cowork client CLI, see [../cowork/](../cowork/).

## Deploy

[![Deploy on Railway](https://railway.com/button.svg)](https://railway.com/new/template?code=systemprompt-gateway)

The template provisions:
- A `gateway` service from `ghcr.io/systempromptio/systemprompt-template:latest`
- A Postgres database (Railway plugin)
- Wiring: `DATABASE_URL` is injected automatically

## Required env vars

Set at least one AI provider key in Railway's Variables UI:

- `ANTHROPIC_API_KEY`
- `OPENAI_API_KEY`
- `GEMINI_API_KEY`

## Custom domain

Railway → service → Settings → Networking → *Generate Domain* or add your own.

## Local reference config

The template config lives at [`deploy/railway/railway.json`](https://github.com/systempromptio/systemprompt-template/blob/main/deploy/railway/railway.json). Fork the repo + `railway init` to customise.

Docs: https://systemprompt.io/documentation/?utm_source=railway&utm_medium=install_doc
