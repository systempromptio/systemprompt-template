# Manage Services

Operate the running Enterprise Demo stack: services, the Postgres database, and scheduled jobs.

## When to Use

Use this skill to bring the stack up, check what is running, inspect the database, watch scheduled jobs, or tail logs while diagnosing an operational issue.

## How to Use

### Services

`infra services` manages the API server, agents, and MCP servers together.

```bash
systemprompt infra services start            # start API, agents, and MCP servers
systemprompt infra services status           # detailed status: which are running, PIDs
systemprompt infra services restart          # restart all services
systemprompt infra services stop             # stop gracefully
systemprompt infra services cleanup          # clear orphaned processes / stale entries
```

### Database

The local stack runs a per-clone Docker Postgres. The CLI exposes status, schema, and migration helpers:

```bash
systemprompt infra db status                 # connection status
systemprompt infra db tables                 # tables with row counts and sizes
systemprompt infra db migrations status      # migration status across extensions
systemprompt infra db query "SELECT ..."     # read-only SQL
systemprompt infra db migrate-repair         # fix migration checksum drift in place (no data loss)
```

For container lifecycle the justfile recipes are authoritative: `just db-up`, `just db-down`, `just db-logs`. There is no destructive reset - recover drift in place with `infra db migrate-repair` (or `just repair-migrations`).

### Jobs

```bash
systemprompt infra jobs list                 # available jobs
systemprompt infra jobs run <job>            # run a scheduled job manually
systemprompt infra jobs history              # execution history
```

The `publish_pipeline` job also runs automatically at server startup.

### Logs while operating

```bash
systemprompt infra logs stream --since 30s            # live tail (alias: follow)
systemprompt infra logs view --level error --since 1h # recent errors
```

### Typical workflow

1. `infra services status` - confirm what is up.
2. `infra db status` - confirm the database is reachable and migrated.
3. `infra services start` - bring up anything missing.
4. `infra logs stream` - watch startup and catch errors.
