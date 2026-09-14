# Local evaluation containers

Run `just evals-up`, then `just evals-probe`. Stop containers with
`just evals-down`; the database volume is retained. These commands use an
isolated Compose project and do not change other running instances.

The harness prefers `/usr/bin/docker`, avoiding the stale WSL `docker.exe`
wrapper. Override it with `EVAL_DOCKER_BIN` when necessary. Build the pinned
client image with `/usr/bin/docker compose -f deploy/evaluator/compose.yaml build
client-probe` if `systemprompt-evaluator:local` is not installed.

The database is reachable only on an internal Docker network. Client probes
have no network, provider credentials, host mounts, or Docker socket. Their
root filesystem is read-only; client state uses size-limited temporary mounts.
The database password is a local fixture value, not a deployment credential.

These are infrastructure probes, not evaluation results. They prove that
PostgreSQL and the pinned native clients start under the container restrictions.
They do not prove inference, MCP access, approvals, publication, or scoring.

Remaining integration work is tracked in [the evaluation architecture and readiness guide](../../docs/evals.md).
