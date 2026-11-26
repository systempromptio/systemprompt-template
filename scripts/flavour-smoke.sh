#!/usr/bin/env bash
# Boot one flavour's image with its own Postgres and assert the smoke
# expectations in flavours/<flavour>/flavour.yaml.
#   scripts/flavour-smoke.sh FLAVOUR [--no-build] [--image IMAGE]
#       [--compose FILE] [--service NAME]
#       [--release [--version V] [--upgrade-from IMAGE]]
# Compose project sp-smoke-<flavour>, an ephemeral host port, isolated volumes,
# always torn down; safe to run for every flavour at once. One line per
# assertion, results in target/flavour-smoke/<flavour>.json, container logs in
# target/flavour-smoke/<flavour>.log. --release adds the release checks: an
# optional in-place upgrade from an older image, users, audit rows and identity
# surviving recreation, the version string and the registration denial.
set -euo pipefail
source "$(dirname "$0")/lib/flavour.sh"
flavour="${1:?usage: flavour-smoke.sh FLAVOUR [--no-build] [--image IMAGE] [--release ...]}"
shift
flavour_require "$flavour"
build=true image="" release=false version="" upgrade_from=""
compose_file="$FLAVOUR_ROOT/docker-compose.yml"
app_service=app
while [ $# -gt 0 ]; do
    case "$1" in
        --no-build) build=false; shift ;;
        --image) image="${2:?}"; build=false; shift 2 ;;
        --compose) compose_file="${2:?}"; shift 2 ;;
        --service) app_service="${2:?}"; shift 2 ;;
        --release) release=true; shift ;;
        --version) version="${2:?}"; shift 2 ;;
        --upgrade-from) upgrade_from="${2:?}"; shift 2 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done
image="${image:-systemprompt-template:$flavour}"
if [ "$build" = true ]; then
    bash "$FLAVOUR_ROOT/scripts/flavour-build.sh" "$flavour" --tag "$image"
fi

out_dir="$FLAVOUR_ROOT/target/flavour-smoke"
mkdir -p "$out_dir"
project="sp-smoke-$flavour"
if [ "$release" = true ]; then project="$project-release-$$"; fi
work="$(mktemp -d)"
results="$work/results.tsv"
: > "$results"
ai_key=""
if [ "$(flavour_get "$flavour" requires_ai_key)" = true ]; then ai_key=sk-ant-smoke-placeholder; fi
cat > "$work/smoke.env" <<ENV
ADMIN_EMAIL=smoke@example.invalid
POSTGRES_PASSWORD=smoketest
ANTHROPIC_API_KEY=$ai_key
OPENAI_API_KEY=
GEMINI_API_KEY=
EXTERNAL_URL=
HTTP_PORT=0
FLAVOUR=$flavour
ENV
compose=(docker compose --env-file "$work/smoke.env" -p "$project" -f "$compose_file" -f "$work/image.yml")

finish() {
    status=$?
    "${compose[@]}" logs --no-color > "$out_dir/$flavour.log" 2>&1 || true
    "${compose[@]}" down -v --remove-orphans > /dev/null 2>&1 || true
    if [ -s "$results" ]; then
        jq -R -s --arg flavour "$flavour" --arg image "$image" '
            split("\n") | map(select(length > 0) | split("\t")
                | {name: .[0], ok: (.[1] == "pass"), detail: .[2]})
            | {flavour: $flavour, image: $image,
               passed: (map(select(.ok)) | length),
               failed: (map(select(.ok | not)) | length),
               assertions: .}' "$results" > "$out_dir/$flavour.json"
        if [ "$status" -eq 0 ] && grep -q $'\tfail\t' "$results"; then status=1; fi
    fi
    if [ "$status" -ne 0 ]; then tail -100 "$out_dir/$flavour.log"; fi
    passed="$(grep -c $'\tpass\t' "$results" || true)"
    failed="$(grep -c $'\tfail\t' "$results" || true)"
    echo "smoke $flavour: $passed passed, $failed failed$( [ "$status" -eq 0 ] || echo ' — FAILED') (target/flavour-smoke/$flavour.json)"
    rm -rf "$work"
    exit "$status"
}
trap finish EXIT

record() {
    printf '%s\t%s\t%s\n' "$1" "$2" "$3" >> "$results"
    printf '%-4s %s %s\n' "$2" "$1" "${3:+($3)}"
}
set_image() {
    flavour_isolated_override "$work/image.yml" "$1" "$app_service" \
        --env-file "$work/smoke.env" -p "$project" -f "$compose_file"
}
http_status() { curl -s -o /dev/null -w '%{http_code}' --max-time 10 "$@" || true; }
wait_ready() {
    local port
    port="$("${compose[@]}" port "$app_service" 8080 | head -1 | tr -d '\r' | sed 's/.*://')"
    base="http://${SMOKE_HOST:-localhost}:$port"
    for _ in $(seq 1 120); do
        if [ "$(docker inspect -f '{{.RestartCount}}' "$("${compose[@]}" ps -q "$app_service")" 2>/dev/null || echo 0)" -gt 0 ]; then
            echo "$image exited during boot" >&2
            record health fail "exited during boot: $("${compose[@]}" logs --no-color --no-log-prefix "$app_service" 2>&1 | sed 's/\x1b\[[0-9;]*[A-Za-z]//g' | grep -m1 -E 'Error|ERROR' | cut -c1-300)"
            return 1
        fi
        # Why: the early-bind listener answers 200 {"status":"starting"} while
        # the server is still bootstrapping, so a bare 200 proves nothing.
        if curl --max-time 3 -fs "$base/api/v1/health" 2>/dev/null | jq -e '.status == "healthy"' > /dev/null 2>&1 \
            && [ "$(http_status "$base/readyz")" = 200 ]; then
            return 0
        fi
        sleep 3
    done
    echo "$image did not become ready" >&2
    record health fail "not ready within 360 s"
    return 1
}
# Why: `compose exec` forwards stdin, so inside a `while read` loop it would
# swallow the rest of the expectation list.
cli() { "${compose[@]}" exec -T "$app_service" /app/bin/systemprompt "$@" < /dev/null; }
psql_app() { "${compose[@]}" exec -T postgres psql -U systemprompt -d systemprompt "$@"; }
users() { psql_app -Atc "SELECT id FROM users WHERE NOT ('anonymous' = ANY(roles)) ORDER BY id"; }
audit() {
    psql_app -Atc "SELECT user_id, trace_id, tokens_used, cost_microdollars FROM ai_requests WHERE id = 'release-smoke-audit'"
}

set_image "${upgrade_from:-$image}"
"${compose[@]}" up -d
wait_ready
record health pass "$base/api/v1/health and /readyz"

if [ "$release" = true ]; then
    before_users="$(users)"
    test -n "$before_users"
    psql_app -v ON_ERROR_STOP=1 <<'SQL'
INSERT INTO ai_requests
    (id, request_id, user_id, context_id, trace_id, provider, model, status,
     input_tokens, output_tokens, tokens_used, cost_microdollars, actor_kind, actor_id)
SELECT 'release-smoke-audit', 'release-smoke-audit', id, 'release-smoke-context', 'release-smoke-trace',
       'anthropic', 'smoke-model', 'completed', 12, 8, 20, 100, 'user', id
FROM users WHERE NOT ('anonymous' = ANY(roles)) ORDER BY id LIMIT 1;
SQL
    before_audit="$(audit)"
    test -n "$before_audit"
    if [ -n "$upgrade_from" ]; then
        set_image "$image"
        "${compose[@]}" up -d --no-deps "$app_service"
        wait_ready
        if [ "$(users)" = "$before_users" ] && [ "$(audit)" = "$before_audit" ]; then
            record upgrade pass "from $upgrade_from"
        else
            record upgrade fail "users or audit rows changed upgrading from $upgrade_from"
        fi
    fi
fi

while IFS= read -r route; do
    [ -n "$route" ] || continue
    code="$(http_status "$base$route")"
    if [ "$code" != 404 ] && [ "$code" -lt 500 ] 2>/dev/null; then
        record "route present $route" pass "$code"
    else
        record "route present $route" fail "$code"
    fi
done < <(flavour_get "$flavour" smoke.routes_present)
while IFS= read -r route; do
    [ -n "$route" ] || continue
    code="$(http_status "$base$route")"
    if [ "$code" = 404 ]; then record "route absent $route" pass "$code"; else record "route absent $route" fail "$code"; fi
done < <(flavour_get "$flavour" smoke.routes_absent)

# A group is a command path: "cloud" is top level, "admin agents" is read
# from `admin --help`.
has_group() {
    local leaf parent
    leaf="$(printf '%s' "$1" | awk '{print $NF}')"
    parent="$(printf '%s' "$1" | awk '{$NF=""; print}')"
    # shellcheck disable=SC2086
    cli $parent --help 2>&1 \
        | awk '/^Commands:/{p=1;next} p&&/^[^ ]/{p=0} p&&NF{print $1}' \
        | grep -qxF "$leaf"
}
while IFS= read -r group; do
    [ -n "$group" ] || continue
    if has_group "$group"; then record "cli group $group" pass ""; else record "cli group $group" fail "not in --help"; fi
done < <(flavour_get "$flavour" smoke.cli_groups_present)
while IFS= read -r group; do
    [ -n "$group" ] || continue
    if has_group "$group"; then record "cli group absent $group" fail "in --help"; else record "cli group absent $group" pass ""; fi
done < <(flavour_get "$flavour" smoke.cli_groups_absent)

capabilities="$( { cli plugins capabilities; cli plugins capabilities jobs; cli plugins capabilities schemas; cli plugins capabilities roles; } 2>&1 || true)"
while IFS= read -r capability; do
    [ -n "$capability" ] || continue
    if printf '%s' "$capabilities" | grep -qF "$capability"; then record "capability $capability" pass ""; else record "capability $capability" fail "not in plugins capabilities"; fi
done < <(flavour_get "$flavour" smoke.capabilities)

while IFS= read -r server; do
    [ -n "$server" ] || continue
    if tools="$(cli plugins mcp tools -s "$server" 2>&1)" && [ -n "$tools" ]; then
        record "mcp tools/list $server" pass "$(printf '%s\n' "$tools" | wc -l | tr -d ' ') lines"
    else
        record "mcp tools/list $server" fail "$(printf '%s' "$tools" | tail -1)"
    fi
done < <(flavour_get "$flavour" smoke.mcp_servers)

landing="$(flavour_get "$flavour" smoke.root_redirect 2>/dev/null || true)"
if [ -n "$landing" ]; then
    root="$(curl -s --max-time 10 -D - "$base/" || true)"
    if printf '%s' "$root" | grep -qiE "^location: $landing\s*$|http-equiv=\"refresh\" content=\"0; url=$landing\""; then
        record "/ sends to $landing" pass ""
    else
        record "/ sends to $landing" fail "$(printf '%s' "$root" | head -1 | tr -d '\r')"
    fi
fi

# Why: the crawl renders what the database holds, and a fresh install holds only
# the admin. One member, a group and a project with that member, and (where the
# gateway is compiled) one gateway request make the row-dependent sections
# render. Every row goes through the CLI or the console API the flavour ships.
# The request is refused by access control (a fresh install grants no model),
# which still records it; a routed request would need a real provider key.
seed_post() {
    local code
    code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 10 -H "Cookie: $cookie" \
        -H 'Content-Type: application/json' -d "$2" "$base/api/public/admin$1" || true)"
    case "$code" in 2??) record "seed $1" pass "$code" ;; *) record "seed $1" fail "$code" ;; esac
}
seed_console() {
    local member key code
    cli admin users create --name smoke-member --email smoke-member@example.invalid --if-not-exists > /dev/null 2>&1 || true
    member="$(cli --json admin users show smoke-member@example.invalid 2>/dev/null | jq -rs '.[0].id // empty' || true)"
    if [ -z "$member" ]; then record "seed member" fail "admin users create/show gave no id"; return; fi
    record "seed member" pass "$member (role user)"
    seed_post /groups '{"id":"smoke-group","name":"Smoke group"}'
    seed_post /groups/smoke-group/members "{\"user_id\":\"$member\"}"
    seed_post /projects '{"id":"smoke-project","name":"Smoke project"}'
    seed_post /projects/smoke-project/members "{\"user_id\":\"$member\"}"
    if [ "$(flavour_get "$flavour" requires_ai_key)" = true ]; then
        key="$(cli admin users api-key issue --user "$member" --name smoke 2>/dev/null \
            | sed 's/\x1b\[[0-9;]*[A-Za-z]//g' | grep -oE 'sp-live-[A-Za-z0-9]+\.[A-Za-z0-9]+' | head -1 || true)"
        code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 30 -H "Authorization: Bearer $key" \
            -H 'Content-Type: application/json' -H 'anthropic-version: 2023-06-01' \
            -d '{"model":"claude-sonnet-4-5","max_tokens":8,"messages":[{"role":"user","content":"smoke"}]}' \
            "$base/v1/messages" || true)"
        if [ -n "$key" ] && [ "$code" -lt 500 ] 2>/dev/null && [ "$code" != 401 ] && [ "$code" != 404 ]; then
            record "seed gateway request" pass "$code"
        else
            record "seed gateway request" fail "api key or request refused: $code"
        fi
    fi
}

pages="$(flavour_get "$flavour" smoke.console_pages 2>/dev/null || true)"
signed_absent="$(flavour_get "$flavour" smoke.routes_absent_signed_in 2>/dev/null || true)"
if [ -n "$pages$signed_absent" ]; then
    link="$(cli admin users webauthn generate-setup-token --email smoke@example.invalid 2>/dev/null \
        | sed 's/\x1b\[[0-9;]*[A-Za-z]//g' | grep -oE 'https?://[^ ]*/auth/link-passkey\?token=[A-Za-z0-9_-]+' | head -1 || true)"
    if [ -n "$link" ] && cookie="$(python3 -I "$FLAVOUR_ROOT/scripts/lib/console-login.py" "$base" "${link%%/auth/*}" \
        smoke@example.invalid "${link##*token=}" 2> "$work/login.err")"; then
        record "console sign-in" pass "passkey and PKCE as the login page runs them"
        while IFS= read -r page; do
            [ -n "$page" ] || continue
            code="$(http_status -H "Cookie: $cookie" "$base$page")"
            if [ "$code" = 200 ]; then record "console page $page" pass "$code"; else record "console page $page" fail "$code"; fi
        done <<< "$pages"
        while IFS= read -r route; do
            [ -n "$route" ] || continue
            code="$(http_status -H "Cookie: $cookie" "$base$route")"
            if [ "$code" = 404 ]; then record "signed-in route absent $route" pass "$code"; else record "signed-in route absent $route" fail "$code"; fi
        done <<< "$signed_absent"
        seed_console
        crawl="$(python3 -I "$FLAVOUR_ROOT/scripts/lib/console-crawl.py" "$base" "$cookie" 2> "$work/crawl.err" || true)"
        if [ -z "$crawl" ]; then record "console nav pages" fail "$(tail -1 "$work/crawl.err")"; fi
        while read -r kind code page; do
            [ -n "$page" ] || continue
            if [ "$kind" = nav ]; then
                if [ "$code" = 200 ]; then record "console nav page $page" pass "$code"; else record "console nav page $page" fail "$code"; fi
            elif [ "$code" != 404 ] && [ "$code" -lt 500 ] 2>/dev/null; then
                record "console link $page" pass "$code"
            else
                record "console link $page" fail "$code"
            fi
        done <<< "$crawl"
        home="$(curl -s --max-time 10 -H "Cookie: $cookie" "$base/admin" || true)"
        missing="$(flavour_py nav-titles "$flavour" | while IFS= read -r title; do
            html="$(printf '%s' "$title" | sed 's/&/\&amp;/g')"
            printf '%s' "$home" | grep -qF "sp-nav-group__title\">$html<" || printf '%s; ' "$title"; done)"
        if [ -z "$missing" ]; then record "console nav" pass "every configured section"; else record "console nav" fail "missing: $missing"; fi
    else
        record "console sign-in" fail "$( [ -n "$link" ] && tail -1 "$work/login.err" || echo 'no setup link from admin users webauthn generate-setup-token')"
    fi
fi

code="$(http_status "$base/v1/models")"
if [ "$(flavour_get "$flavour" smoke.models_endpoint)" = true ]; then
    if [ "$code" != 404 ] && [ "$code" -lt 500 ] 2>/dev/null; then record "/v1/models present" pass "$code"; else record "/v1/models present" fail "$code"; fi
else
    if [ "$code" = 404 ]; then record "/v1/models absent" pass "$code"; else record "/v1/models absent" fail "$code"; fi
fi

if [ "$release" = true ]; then
    actual="$(cli --version)"
    if [ -z "$version" ] || [[ "$actual" =~ (^|[[:space:]])$version($|[[:space:]]) ]]; then
        record version pass "$actual"
    else
        record version fail "$actual, expected $version"
    fi
    before_identity="$("${compose[@]}" exec -T "$app_service" sha256sum /app/.systemprompt/profiles/docker/secrets.json)"
    if [ "$app_service" = systemprompt ]; then
        "${compose[@]}" exec -T "$app_service" sh -c 'mkdir -p /app/storage/files/uploads /app/storage/files/images/generated; echo retained > /app/storage/files/uploads/release-smoke; echo retained > /app/storage/files/images/generated/release-smoke'
    fi
    # shellcheck disable=SC2016
    if "${compose[@]}" exec -T "$app_service" sh -c 'test -n "$(find /app/storage/files/admin -name "*.hbs" -print -quit)"'; then
        record "admin templates shipped" pass ""
    else
        record "admin templates shipped" fail "no .hbs under /app/storage/files/admin"
    fi
    "${compose[@]}" up -d --force-recreate --no-deps "$app_service"
    wait_ready
    if [ "$(users)" = "$before_users" ] && [ "$(audit)" = "$before_audit" ] \
        && [ "$("${compose[@]}" exec -T "$app_service" sha256sum /app/.systemprompt/profiles/docker/secrets.json)" = "$before_identity" ]; then
        record "state survives recreation" pass "users, audit rows, identity"
    else
        record "state survives recreation" fail "users, audit rows or identity changed"
    fi
    if [ "$app_service" = systemprompt ]; then
        for file in /app/storage/files/uploads/release-smoke /app/storage/files/images/generated/release-smoke; do
            if [ "$("${compose[@]}" exec -T "$app_service" cat "$file")" = retained ]; then record "retained $file" pass ""; else record "retained $file" fail ""; fi
        done
    fi
    code="$(http_status -H "Content-Type: application/json" -d '{"name":"Unauthorized","email":"unauthorized@example.invalid","role":"admin"}' "$base/admin/api/register")"
    if [ "$code" = 403 ]; then record "registration denied" pass "$code"; else record "registration denied" fail "$code"; fi
fi
