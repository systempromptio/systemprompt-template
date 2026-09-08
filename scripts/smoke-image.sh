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
app_service="${SMOKE_APP_SERVICE:-app}"
compose_file="${SMOKE_COMPOSE_FILE:-$root/deploy/compose/one-click.docker-compose.yml}"
# An explicit env file also works when a WSL wrapper invokes docker.exe,
# which does not inherit ordinary Linux environment exports.
cat > "$work/smoke.env" <<'ENV'
ADMIN_EMAIL=smoke@example.invalid
POSTGRES_PASSWORD=smoketest
ANTHROPIC_API_KEY=sk-ant-smoke-placeholder
OPENAI_API_KEY=
GEMINI_API_KEY=
EXTERNAL_URL=
HTTP_PORT=0
ENV
compose=(docker compose --env-file "$work/smoke.env" -p "$project" -f "$compose_file" -f "$work/image.yml")
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
    python3 - "$1" "$work/image.yml" "$app_service" <<'PY'
import json,sys
service = {'image': sys.argv[1]}
if sys.argv[3] == 'systemprompt':
    service['ports'] = ['127.0.0.1::8080']
json.dump({'services': {sys.argv[3]: service}}, open(sys.argv[2], 'w'))
PY
}
wait_health() {
    local port
    port=$("${compose[@]}" port "$app_service" 8080 | head -1 | tr -d '\r' | sed 's/.*://')
    for _ in $(seq 1 120); do
        if curl --max-time 3 -fs "http://${SMOKE_HOST:-localhost}:$port/api/v1/health" >/dev/null && \
            curl --max-time 3 -fs "http://${SMOKE_HOST:-localhost}:$port/" >/dev/null; then
            base="http://${SMOKE_HOST:-localhost}:$port"
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
before_identity=$("${compose[@]}" exec -T "$app_service" sha256sum /app/.systemprompt/profiles/docker/secrets.json)
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
    "${compose[@]}" up -d --no-deps "$app_service"
    wait_health
    test "$(users)" = "$before_users"
    test "$(audit)" = "$before_audit"
fi
before_identity=$("${compose[@]}" exec -T "$app_service" sha256sum /app/.systemprompt/profiles/docker/secrets.json)
if [ "$app_service" = systemprompt ]; then
    "${compose[@]}" exec -T "$app_service" sh -c 'mkdir -p /app/storage/files/uploads /app/storage/files/images/generated; echo retained > /app/storage/files/uploads/release-smoke; echo retained > /app/storage/files/images/generated/release-smoke'
fi
actual=$("${compose[@]}" exec -T "$app_service" /app/bin/systemprompt --version)
echo "$actual"
if [ -n "$version" ]; then
    [[ "$actual" =~ (^|[[:space:]])$version($|[[:space:]]) ]]
fi
# A healthy API alone does not prove runtime templates and assets were shipped.
curl -fsSL "$base/" > "$work/index.html"
test -s "$work/index.html"
"${compose[@]}" exec -T "$app_service" test -s /app/storage/files/admin/templates/layout.hbs || \
    "${compose[@]}" exec -T "$app_service" sh -c 'test -n "$(find /app/storage/files/admin -name "*.hbs" -print -quit)"'
"${compose[@]}" exec -T "$app_service" test -x /app/bin/systemprompt-mcp-agent
"${compose[@]}" exec -T "$app_service" test -f /app/extensions/mcp/systemprompt/manifest.yaml
"${compose[@]}" up -d --force-recreate --no-deps "$app_service"
wait_health
test "$(users)" = "$before_users"
test "$(audit)" = "$before_audit"
test "$("${compose[@]}" exec -T "$app_service" sha256sum /app/.systemprompt/profiles/docker/secrets.json)" = "$before_identity"
if [ "$app_service" = systemprompt ]; then
    for file in /app/storage/files/uploads/release-smoke /app/storage/files/images/generated/release-smoke; do
        test "$("${compose[@]}" exec -T "$app_service" cat "$file")" = retained
    done
fi
status=$(curl -s -o /dev/null -w "%{http_code}" -H "Content-Type: application/json" -d '{"name":"Unauthorized","email":"unauthorized@example.invalid","role":"admin"}' "$base/admin/api/register")
test "$status" = 403
echo "Image boot, assets, identity persistence, registration denial and recreation passed: $image"
