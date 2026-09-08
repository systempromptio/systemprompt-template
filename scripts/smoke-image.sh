#!/usr/bin/env bash
# Boot an exact image with isolated Postgres and persistent application volumes.
# Optional third argument boots an older image first to exercise in-place upgrade.
set -euo pipefail
image="${1:?usage: smoke-image.sh IMAGE [VERSION] [UPGRADE_FROM]}"
version="${2:-}"
upgrade_from="${3:-}"
root="$(cd "$(dirname "$0")/.." && pwd)"
project="release-smoke-$$"
work="$(mktemp -d)"
export ADMIN_EMAIL=smoke@example.invalid POSTGRES_PASSWORD=smoketest
export ANTHROPIC_API_KEY=sk-ant-smoke-placeholder OPENAI_API_KEY= GEMINI_API_KEY=
export HTTP_PORT=0
compose=(docker compose -p "$project" -f "$root/deploy/compose/one-click.docker-compose.yml" -f "$work/image.yml")
cleanup() {
    status=$?
    "${compose[@]}" logs --no-color > "$work/containers.log" 2>&1 || true
    mkdir -p "$root/.build/smoke"
    cp "$work/containers.log" "$root/.build/smoke/$project.log"
    if [ "$status" -ne 0 ]; then tail -100 "$work/containers.log"; fi
    "${compose[@]}" down -v --remove-orphans >/dev/null 2>&1 || true
    rm -rf "$work"
    exit "$status"
}
trap cleanup EXIT
set_image() {
    # JSON is valid YAML and quotes registry references without shell expansion.
    python3 - "$1" "$work/image.yml" <<'PY'
import json,sys
json.dump({'services': {'app': {'image': sys.argv[1]}}}, open(sys.argv[2], 'w'))
PY
}
wait_health() {
    local port
    port=$("${compose[@]}" port app 8080 | head -1 | sed 's/.*://')
    for _ in $(seq 1 120); do
        if curl -fs "http://localhost:$port/api/v1/health" >/dev/null && \
            curl -fs "http://localhost:$port/" >/dev/null; then
            base="http://localhost:$port"
            return
        fi
        sleep 3
    done
    echo "Image did not become healthy" >&2
    return 1
}
users() {
    "${compose[@]}" exec -T postgres psql -U systemprompt -d systemprompt -Atc \
        "SELECT id FROM users ORDER BY id"
}
set_image "${upgrade_from:-$image}"
"${compose[@]}" up -d
wait_health
before_users=$(users)
test -n "$before_users"
"${compose[@]}" exec -T postgres psql -v ON_ERROR_STOP=1 -U systemprompt -d systemprompt <<'SQL'
INSERT INTO ai_requests
    (id, request_id, user_id, context_id, trace_id, provider, model, status,
     input_tokens, output_tokens, tokens_used, cost_microdollars, actor_kind, actor_id)
SELECT 'release-smoke-audit', 'release-smoke-audit', id, 'release-smoke-context', 'release-smoke-trace',
       'anthropic', 'smoke-model', 'completed', 12, 8, 20, 100, 'user', id
FROM users ORDER BY id LIMIT 1;
SQL
audit() {
    "${compose[@]}" exec -T postgres psql -U systemprompt -d systemprompt -Atc \
        "SELECT user_id, trace_id, tokens_used, cost_microdollars FROM ai_requests WHERE id = 'release-smoke-audit'"
}
before_audit=$(audit)
test -n "$before_audit"
if [ -n "$upgrade_from" ]; then
    set_image "$image"
    "${compose[@]}" up -d --no-deps app
    wait_health
    test "$(users)" = "$before_users"
    test "$(audit)" = "$before_audit"
fi
actual=$("${compose[@]}" exec -T app /app/bin/systemprompt --version)
echo "$actual"
if [ -n "$version" ]; then
    [[ "$actual" =~ (^|[[:space:]])$version($|[[:space:]]) ]]
fi
# A healthy API alone does not prove runtime templates and assets were shipped.
curl -fsSL "$base/" > "$work/index.html"
test -s "$work/index.html"
"${compose[@]}" exec -T app test -s /app/storage/files/admin/templates/layout.hbs || \
    "${compose[@]}" exec -T app sh -c 'test -n "$(find /app/storage/files/admin -name "*.hbs" -print -quit)"'
"${compose[@]}" exec -T app test -x /app/bin/systemprompt-mcp-agent
"${compose[@]}" exec -T app test -f /app/extensions/mcp/systemprompt/manifest.yaml
"${compose[@]}" restart app
wait_health
test "$(users)" = "$before_users"
    test "$(audit)" = "$before_audit"
echo "Image boot, assets, identity persistence and restart passed: $image"
