# Deploy the gateway on Northflank

Deploys the `systemprompt-gateway` server on [Northflank](https://northflank.com) as a stack template (gateway service + managed Postgres addon + persistent volume).

## One-click install

[![Deploy on Northflank](https://assets.northflank.com/deploy_to_northflank_smm_36700fb050.svg)](https://app.northflank.com/s/account/templates/new?data=6a58eb70982d53bd314abce3)

1. Click the button. The `systemprompt-io` template opens pre-filled in your Northflank account; save it.
2. Add at least one of `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` / `GEMINI_API_KEY` as an argument override (the gateway refuses to boot without a provider key).
3. Run the template. The workflow provisions a Postgres addon, deploys the gateway with a public HTTPS endpoint on port 8080, links a secret group that wires `DATABASE_URL` from the addon's admin connection string and `EXTERNAL_URL` from the generated domain, and attaches a persistent volume for web assets, storage, and profile state.
4. First boot runs migrations and the publish pipeline; the deploy goes healthy in about 2 minutes. The gateway may restart once or twice while secrets and the volume land; this is expected.
5. Verify at `https://<generated>.code.run/api/v1/health`, then open the domain root for the admin UI.

## Manual path

Create a template yourself: **Templates** → **Create template** → paste [`deploy/northflank/template.json`](https://github.com/systempromptio/systemprompt-template/blob/main/deploy/northflank/template.json), then follow steps 2-5 above.

## Notes

- Plans default to `nf-compute-20` (fits the free developer allowance); resize the addon and gateway in your project after deploy if you need more headroom.
- The volume pins the gateway to a single instance (`ReadWriteOnce`).

Docs: https://systemprompt.io/documentation/?utm_source=northflank&utm_medium=install_doc
