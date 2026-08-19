# Demo Harness

Setup and seeding scripts for a local Astound Digital instance. The generic
template demo catalogue (governance, analytics, agents, scenarios, recordings)
was removed when this fork narrowed to the Salesforce + Claude Code use case;
what remains is the working harness other tooling depends on.

## Scripts

| Script | Purpose |
|--------|---------|
| `00-preflight.sh` | Verifies services are up, mints and saves an admin token to `demo/.token` (gitignored; a user-scope token lands in `demo/.token.user`). |
| `01-seed-data.sh` | Seeds demo analytics/log/trace data and uploads fixtures from `fixtures/`. |
| `sweep.sh` | Runs every `demo/**/*.sh` script it finds (excluding the harness itself) and prints a pass/fail summary. |
| `_common.sh` | Shared helpers sourced by the scripts above (token handling, output formatting, governance-disabled notes). |

`fixtures/` holds the files `01-seed-data.sh` uploads.

## Usage

```bash
./demo/00-preflight.sh   # check services, acquire token
./demo/01-seed-data.sh   # seed data
```

`just benchmark` reads the token from `demo/.token`.

Read `AGENTS.md` before driving these scripts from an agent.
