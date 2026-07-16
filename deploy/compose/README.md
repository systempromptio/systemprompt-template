# Canonical one-click compose

`one-click.docker-compose.yml` is the deployment artifact referenced by
marketplace catalogs (Portainer app templates, Dokploy blueprints, CasaOS,
and the donors for CapRover/Elestio variants). It differs from the root
`docker-compose.yml` in exactly two ways:

- it pulls the published GHCR image instead of building from source;
- it does not publish the Postgres port (catalog installs have no reason to
  expose the database).

The service and volume topology must stay identical to the root compose —
the `compose-drift` job in `.github/workflows/smoke-tests.yml` diffs the two
on every release.

## Environment

| Variable | Required | Notes |
|---|---|---|
| `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` / `GEMINI_API_KEY` | at least one | Blank values are treated as unset by the entrypoint. |
| `POSTGRES_PASSWORD` | recommended | Defaults to `systemprompt`. |
| `EXTERNAL_URL` | optional | Public URL of the deployment; sets `api_external_url` and CORS. |
| `HTTP_PORT` | optional | Host port (default 8080). |
| `HOST` | optional | Bind address inside the container (default `0.0.0.0`; `::` on IPv6-only networks). |

First boot authors a profile, runs migrations, bootstraps the admin user, and
runs the publish pipeline — allow several minutes before `/api/v1/health`
returns 200 (the healthcheck uses a 300s start period).
