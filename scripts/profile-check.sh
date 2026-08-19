#!/usr/bin/env bash
# Validate one .systemprompt profile without needing a running server.
#
# Usage: profile-check.sh <profile> [--live]
#
# Checks, in order: profile files exist, secrets.json parses and carries the
# required keys, profile.yaml URLs look sane, the database in secrets.json is
# reachable from THIS machine, and (when a CLI binary exists) that the profile
# actually deserializes — the loader rejects unknown YAML keys, so this is the
# authoritative schema check. --live additionally probes the external URL's
# health endpoint. Portable across macOS and Linux (no GNU-only flags).
set -euo pipefail

PROFILE="${1:?usage: profile-check.sh <profile> [--live]}"
LIVE=0
[ "${2:-}" = "--live" ] && LIVE=1

DIR=".systemprompt/profiles/$PROFILE"
YAML="$DIR/profile.yaml"
SECRETS="$DIR/secrets.json"

PASS=0 WARN=0 FAIL=0
ok()   { printf '  \033[32mPASS\033[0m  %s\n' "$1"; PASS=$((PASS+1)); }
warn() { printf '  \033[33mWARN\033[0m  %s\n' "$1"; WARN=$((WARN+1)); }
bad()  { printf '  \033[31mFAIL\033[0m  %s\n' "$1"; FAIL=$((FAIL+1)); }

yaml_val() {
    sed -n "s/^[[:space:]]*$1:[[:space:]]*//p" "$YAML" | head -1 | tr -d '"' | tr -d "'"
}

echo "── profile: $PROFILE ──"

if [ ! -d "$DIR" ]; then
    bad "$DIR does not exist. Available: $(ls .systemprompt/profiles 2>/dev/null | tr '\n' ' ')"
    exit 1
fi
[ -f "$YAML" ]    && ok "profile.yaml present"    || bad "profile.yaml missing"
[ -f "$SECRETS" ] && ok "secrets.json present"    || bad "secrets.json missing"
[ "$FAIL" -gt 0 ] && { echo "── $PASS pass, $WARN warn, $FAIL fail ──"; exit 1; }

case "$(ls -l "$SECRETS" | cut -c8)" in
    r) warn "secrets.json is world-readable — chmod 600 $SECRETS" ;;
    *) ok "secrets.json permissions" ;;
esac

echo "── secrets ──"
if ! python3 -c "import json,sys; json.load(open('$SECRETS'))" 2>/dev/null; then
    bad "secrets.json is not valid JSON"
    echo "── $PASS pass, $WARN warn, $FAIL fail ──"; exit 1
fi
ok "secrets.json parses"

DB_URL="$(python3 -c "import json; print(json.load(open('$SECRETS')).get('database_url') or '')")"
if [ -n "$DB_URL" ]; then
    ok "database_url set"
else
    bad "database_url missing or empty — the server cannot boot without it"
fi

HAS_AI="$(python3 -c "
import json
s = json.load(open('$SECRETS'))
print(any(s.get(k) for k in ('anthropic','openai','gemini','cerebras')))")"
[ "$HAS_AI" = "True" ] && ok "at least one AI provider key set" \
    || warn "no AI provider key (anthropic/openai/gemini/cerebras) — inference will fail"

echo "── profile.yaml ──"
NAME="$(yaml_val name)"
[ "$NAME" = "$PROFILE" ] && ok "name matches directory ($NAME)" \
    || warn "name is '$NAME' but directory is '$PROFILE'"

TARGET="$(yaml_val target)"
[ -n "$TARGET" ] && ok "target: $TARGET" || warn "no target: field"

EXT_URL="$(yaml_val api_external_url)"
if [ -n "$EXT_URL" ]; then
    case "$EXT_URL" in
        https://*) ok "api_external_url: $EXT_URL" ;;
        http://localhost*|http://127.0.0.1*) ok "api_external_url: $EXT_URL (local)" ;;
        http://*) warn "api_external_url is plain http: $EXT_URL — use https for anything non-local" ;;
        *) bad "api_external_url has no scheme: $EXT_URL" ;;
    esac
else
    bad "api_external_url missing"
fi

PORT="$(yaml_val port)"
case "$PORT" in
    ''|*[!0-9]*) warn "server.port not found or not numeric ('$PORT')" ;;
    *) ok "server.port: $PORT" ;;
esac

echo "── database reachability (from this machine) ──"
if [ -z "$DB_URL" ]; then
    warn "skipped — no database_url"
else
    eval "$(python3 -c "
from urllib.parse import urlparse
u = urlparse('$DB_URL')
print(f'DB_HOST={u.hostname or \"\"} DB_PORT={u.port or 5432} DB_NAME={(u.path or \"/\")[1:]}')")"
    case "$DB_HOST" in
        *.internal|*.flycast)
            warn "DB host $DB_HOST is on a private cloud network — unreachable from dev BY DESIGN; verify with: systemprompt cloud db status --profile $PROFILE" ;;
        '')
            bad "database_url has no host" ;;
        *)
            if command -v pg_isready >/dev/null 2>&1; then
                if pg_isready -h "$DB_HOST" -p "$DB_PORT" -t 5 >/dev/null 2>&1; then
                    ok "pg_isready: $DB_HOST:$DB_PORT accepting connections"
                else
                    bad "pg_isready: $DB_HOST:$DB_PORT not reachable — check the DB service, then cloud firewall ingress (OCI security list / NSG), then VM firewall (firewalld/iptables), then sslmode"
                fi
            elif (exec 3<>"/dev/tcp/$DB_HOST/$DB_PORT") 2>/dev/null; then
                exec 3<&- 3>&-
                ok "tcp: $DB_HOST:$DB_PORT open (install postgresql client for a real check)"
            else
                bad "tcp: $DB_HOST:$DB_PORT closed — check the DB service, then cloud firewall ingress (OCI security list / NSG), then VM firewall, then the port"
            fi
            if command -v psql >/dev/null 2>&1 && [ "$FAIL" -eq 0 ]; then
                if psql "$DB_URL" -Atc 'select 1' >/dev/null 2>&1; then
                    ok "psql auth: select 1 succeeded against $DB_NAME"
                else
                    bad "psql auth failed — check user/password/database name/sslmode in database_url (managed Postgres often needs sslmode=require)"
                fi
            fi ;;
    esac
fi

echo "── CLI deserialization (rejects unknown YAML keys) ──"
BIN=""
for c in target/release/systemprompt target/debug/systemprompt; do
    [ -x "$c" ] && { BIN="$c"; break; }
done
if [ -n "$BIN" ]; then
    if OUT="$("$BIN" cloud profile show "$PROFILE" 2>&1 >/dev/null)"; then
        ok "profile loads cleanly via $BIN"
    else
        bad "profile failed to load: $(echo "$OUT" | head -3 | tr '\n' ' ')"
    fi
else
    warn "no CLI binary — run 'just build' for the authoritative schema check"
fi

if [ "$LIVE" -eq 1 ] && [ -n "$EXT_URL" ]; then
    echo "── live probe ──"
    if curl -fsk --max-time 10 "$EXT_URL/api/v1/health" >/dev/null 2>&1; then
        ok "$EXT_URL/api/v1/health responds"
    else
        bad "$EXT_URL/api/v1/health not responding — server down, port not exposed, or DNS/proxy misconfigured"
    fi
fi

echo "── $PASS pass, $WARN warn, $FAIL fail ──"
[ "$FAIL" -eq 0 ]
