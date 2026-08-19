# systemprompt Documentation

User-facing docs for the gateway published out of this repository.

## The product

| | What it is | Where it runs | Tag series |
|---|---|---|---|
| **systemprompt-gateway** (server) | AI governance gateway — Rust HTTP server + Postgres + MCP extensions | Docker / Linux VM | `v*` |

Clients authenticate with a personal access token issued on
`/admin/access/tokens`.

Deployment goes through the CLI (`just deploy` → `systemprompt cloud deploy`);
the per-platform install recipes the generic template shipped were removed when
this fork narrowed to the Salesforce + Claude Code use case.

Maintainers: the release process (versioning, tag scheme, retention, rollback) is documented in [RELEASING.md](RELEASING.md).

### Running a second clone side-by-side

`just setup-local` accepts port overrides after the three key positions. To run a second clone on HTTP 8081 and Postgres 5433:

```bash
just setup-local <anthropic_key> "" "" 8081 5433
```

### Gateway configuration

- [gateway-routes.md](gateway-routes.md): `/v1/messages` provider routing, CLI route configuration, route access control, and the extensible provider registry.
