#!/usr/bin/env bash
# Isolated local evaluator infrastructure. Never forwards host provider secrets.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
docker_cli="${EVAL_DOCKER_BIN:-/usr/bin/docker}"
if [[ ! -x "$docker_cli" ]]; then
    docker_cli="$(command -v docker)"
fi
compose=("$docker_cli" compose -f "$repo_root/deploy/evaluator/compose.yaml")
case "${1:-}" in
    up)
        "$docker_cli" version --format 'Docker server {{.Server.Version}}'
        "${compose[@]}" up -d --wait --wait-timeout 90 postgres
        ;;
    probe)
        "${compose[@]}" run --rm --no-deps client-probe claude --version
        "${compose[@]}" run --rm --no-deps client-probe opencode --version
        "${compose[@]}" run --rm --no-deps client-probe node -e "$(cat "$repo_root/scripts/evaluator/probe.js")"
        "${compose[@]}" exec -T postgres psql -U evaluator -d evaluator \
            -v ON_ERROR_STOP=1 -c 'SELECT version();'
        ;;
    relay)
        # Why: the relay image copies a release binary, so the crate is built
        # here rather than assumed; the upstream is the gateway this clone runs.
        cargo build --release -p systemprompt-evaluator-relay
        SYSTEMPROMPT_RELAY_UPSTREAM="${SYSTEMPROMPT_RELAY_UPSTREAM:-http://host.docker.internal:8080}" \
            "${compose[@]}" --profile relay up -d --build relay
        ;;
    down)
        "${compose[@]}" --profile relay down
        ;;
    *)
        echo 'Usage: scripts/evaluator/local.sh {up|probe|relay|down}' >&2
        exit 2
        ;;
esac
