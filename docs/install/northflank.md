# Deploy the gateway on Northflank

Deploys the `systemprompt-gateway` server on [Northflank](https://northflank.com) as a stack template (gateway service + managed Postgres addon).

## Install

Once the template is published, use the one-click deploy link (will land here). Manual path:

1. Create a template: **Templates** → **Create template** → paste [`deploy/northflank/template.json`](https://github.com/systempromptio/systemprompt-template/blob/main/deploy/northflank/template.json).
2. Run it, supplying at least one of `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` / `GEMINI_API_KEY` as arguments (the gateway refuses to boot without a provider key).
3. The workflow provisions a Postgres addon, deploys the gateway with a public HTTPS endpoint on port 8080, links a secret group that wires `DATABASE_URL` from the addon's connection details and `EXTERNAL_URL` from the generated domain, and attaches a persistent volume for web assets, storage, and profile state. First boot runs migrations and the publish pipeline — allow several minutes before `/api/v1/health` goes green. The gateway may restart a couple of times in the first minutes while secrets and the volume land; this is expected.

Docs: https://systemprompt.io/documentation/?utm_source=northflank&utm_medium=install_doc
