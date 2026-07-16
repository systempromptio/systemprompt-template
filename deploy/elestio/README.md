# Elestio stack payload

Hand-off package for Elestio's team (they build and operate managed stacks
internally — there is no self-serve submission).

- `docker-compose.yml` — copy of the canonical one-click compose
  (`deploy/compose/one-click.docker-compose.yml`). Elestio's reverse proxy
  terminates TLS; map their injected domain into `EXTERNAL_URL`.
- Required env: at least one of `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` /
  `GEMINI_API_KEY` (collected from the user at deploy time), plus a generated
  `POSTGRES_PASSWORD`.
- Health: `GET /api/v1/health` on port 8080; first boot runs migrations and a
  publish pipeline (allow 5 minutes).
- Licence: BSL 1.1 (source-available). Elestio hosts comparable fair-code
  software (e.g. n8n).
- Assets: icon https://raw.githubusercontent.com/systempromptio/systemprompt-template/main/storage/files/images/icon-256.png,
  docs https://systemprompt.io/documentation/
