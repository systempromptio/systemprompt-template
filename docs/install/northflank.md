# Deploy the gateway on Northflank

Deploys the `systemprompt-gateway` server on [Northflank](https://northflank.com) as a stack template (gateway service + managed Postgres addon).

## Install

Once the template is published, use the one-click deploy link (will land here). Manual path:

1. Create a template: **Templates** → **Create template** → paste [`deploy/northflank/template.json`](https://github.com/systempromptio/systemprompt-template/blob/main/deploy/northflank/template.json).
2. Run it, supplying at least one of `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` / `GEMINI_API_KEY` as arguments (the gateway refuses to boot without a provider key).
3. The workflow provisions a Postgres addon, wires `DATABASE_URL` from its connection details, and deploys the gateway with a public HTTPS endpoint on port 8080. First boot runs migrations and the publish pipeline — allow several minutes before `/api/v1/health` goes green.

Docs: https://systemprompt.io/documentation/?utm_source=northflank&utm_medium=install_doc
