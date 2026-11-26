#!/bin/sh
# Container entrypoint for systemprompt-template.
# Authors a profile via `systemprompt admin setup` on first boot,
# waits for Postgres, runs migrations, starts the server. The image's flavour
# (Dockerfile ARGs) decides whether a provider key is required
# (FLAVOUR_REQUIRES_AI_KEY) and whether the site or only the console assets
# are published (FLAVOUR_SITE).
set -eu
umask 077
profile_created=false

# Railway mounts volumes as root. Prepare ownership, then run the gateway
# under the same unprivileged account used on other container hosts.
if [ "$(id -u)" = 0 ] && [ -n "${SYSTEMPROMPT_DATA_DIR:-}" ]; then
    mkdir -p "$SYSTEMPROMPT_DATA_DIR"
    chown -R app:app "$SYSTEMPROMPT_DATA_DIR"
    exec gosu app "$0" "$@"
fi

# One-click platforms (Railway et al.) export unfilled template variables as
# empty strings; admin setup would record "" as a configured provider key.
# Treat blank as unset.
[ -n "${ANTHROPIC_API_KEY:-}" ] || unset ANTHROPIC_API_KEY
[ -n "${OPENAI_API_KEY:-}" ] || unset OPENAI_API_KEY
[ -n "${GEMINI_API_KEY:-}" ] || unset GEMINI_API_KEY
[ -n "${GITHUB_TOKEN:-}" ] || unset GITHUB_TOKEN
[ -n "${EXTERNAL_URL:-}" ] || unset EXTERNAL_URL

# Platform-neutral external URL. Render injects RENDER_EXTERNAL_URL; every
# other catalog template sets EXTERNAL_URL explicitly.
EXTERNAL_URL="${EXTERNAL_URL:-${RENDER_EXTERNAL_URL:-}}"

# The administrator's address. SYSTEMPROMPT_ADMIN_EMAIL is the name the other
# systemprompt images read; ADMIN_EMAIL is this template's historical name and
# every one-click catalog still sets it. The prefixed one wins when both are set.
[ -n "${SYSTEMPROMPT_ADMIN_EMAIL:-}" ] || unset SYSTEMPROMPT_ADMIN_EMAIL
[ -n "${ADMIN_EMAIL:-}" ] || unset ADMIN_EMAIL
ADMIN_EMAIL="${SYSTEMPROMPT_ADMIN_EMAIL:-${ADMIN_EMAIL:-}}"

if [ -n "${SYSTEMPROMPT_DATA_DIR:-}" ]; then
    python3 /app/container-state.py attach
fi

postgres_ready=false
wait_for_postgres() {
    # Probe DATABASE_URL directly when provided (managed Postgres, e.g. Render);
    # fall back to the compose-style host/user/db vars otherwise.
    if [ -n "${DATABASE_URL:-}" ]; then
        pg_probe() { python3 /app/container-state.py postgres-ready "$DATABASE_URL"; }
        echo "Waiting for Postgres at DATABASE_URL host..."
    else
        PG_HOST="${PG_HOST:-postgres}"
        PG_USER="${PG_USER:-systemprompt}"
        PG_DB="${PG_DB:-systemprompt}"
        pg_probe() { python3 /app/container-state.py postgres-ready "postgres://$PG_USER@$PG_HOST:5432/$PG_DB"; }
        echo "Waiting for Postgres at ${PG_HOST}..."
    fi
    i=0
    until pg_probe >/dev/null 2>&1; do
        i=$((i + 1))
        if [ "$i" -ge 300 ]; then
            echo "ERROR: Postgres did not become ready within 300s." >&2
            exit 1
        fi
        sleep 1
    done
    echo "Postgres is ready."
    postgres_ready=true
}

PROFILE_DIR="${SYSTEMPROMPT_PROFILE_DIR:-/app/.systemprompt/profiles/docker}"
PROFILE_FILE="$PROFILE_DIR/profile.yaml"
SECRETS_FILE="$PROFILE_DIR/secrets.json"

if [ -n "${SYSTEMPROMPT_PROFILE_DIR:-}" ]; then
    # A profile directory was supplied (e.g. bind-mounted air-gap profile).
    # Do not generate anything — just validate the expected files exist.
    if [ ! -f "$PROFILE_FILE" ]; then
        echo "ERROR: SYSTEMPROMPT_PROFILE_DIR is set but $PROFILE_FILE is missing." >&2
        exit 1
    fi
    if [ ! -f "$SECRETS_FILE" ]; then
        echo "ERROR: SYSTEMPROMPT_PROFILE_DIR is set but $SECRETS_FILE is missing." >&2
        exit 1
    fi
    # Why: helm and the air-gap scenario mount this directory read-only and
    # share it between replicas, so nothing below may mint into it. A key
    # minted per container would seal records the other replicas cannot open
    # and sign tokens they reject, and would be lost on every restart.
    # A supplied profile names its own database; the readiness probe below
    # must not fall back to the compose-only `postgres` hostname.
    if [ -z "${DATABASE_URL:-}" ]; then
        DATABASE_URL="$(jq -r '.database_url // empty' "$SECRETS_FILE")"
        if [ -z "$DATABASE_URL" ]; then
            echo "ERROR: $SECRETS_FILE has no database_url and DATABASE_URL is unset." >&2
            exit 1
        fi
    fi
    if [ -z "$(jq -r '.encryption_master_key // empty' "$SECRETS_FILE")" ]; then
        echo "ERROR: $SECRETS_FILE has no encryption_master_key." >&2
        echo "  Core 0.62 refuses to boot without it. Add 64 hex characters" >&2
        echo "  (openssl rand -hex 32) to the shared secrets and redeploy." >&2
        exit 1
    fi
    # Multi-node deployments share one signing key through the
    # `signing_key_pem` secret; a single-node supplied profile may instead
    # point `security.signing_key_path` at a key file it ships.
    if [ -z "$(jq -r '.signing_key_pem // empty' "$SECRETS_FILE")" ]; then
        key_file="$(python3 -c 'import sys, yaml
from pathlib import Path
profile = Path(sys.argv[1])
key = (yaml.safe_load(profile.read_text()) or {}).get("security", {}).get("signing_key_path") or "/app/signing_key.pem"
path = Path(key)
print(path if path.is_absolute() else profile.parent / path)' "$PROFILE_FILE")"
        if [ ! -s "$key_file" ]; then
            echo "ERROR: $SECRETS_FILE has no signing_key_pem and $key_file does not exist." >&2
            echo "  Supply the shared signing key as signing_key_pem (base64 PEM)." >&2
            exit 1
        fi
    fi
else
    if [ "${FLAVOUR_REQUIRES_AI_KEY:-true}" = true ]; then
        if [ -z "${ANTHROPIC_API_KEY:-}" ] && [ -z "${OPENAI_API_KEY:-}" ] && [ -z "${GEMINI_API_KEY:-}" ]; then
            echo "ERROR: set at least one of ANTHROPIC_API_KEY, OPENAI_API_KEY, GEMINI_API_KEY in .env" >&2
            exit 1
        fi
    else
        unset ANTHROPIC_API_KEY OPENAI_API_KEY GEMINI_API_KEY
    fi
    if [ -z "${DATABASE_URL:-}" ]; then
        echo "ERROR: DATABASE_URL is required." >&2
        exit 1
    fi

    if [ ! -f "$PROFILE_FILE" ]; then
        echo "Generating profile via admin setup..."
        # Default provider = first configured key (setup picks up the
        # ANTHROPIC/OPENAI/GEMINI_API_KEY env vars itself); a flavour without
        # AI names none.
        set --
        if [ -n "${ANTHROPIC_API_KEY:-}" ]; then set -- --default-provider anthropic
        elif [ -n "${OPENAI_API_KEY:-}" ]; then set -- --default-provider openai
        elif [ -n "${GEMINI_API_KEY:-}" ]; then set -- --default-provider gemini
        fi
        # Why: core requires --admin-email since 0.41.0. It refuses to invent
        # one because the address is shown as the operator's identity on the
        # device-link consent screen, directly above the control that mints a
        # durable personal access token. Ask here, with the variable named,
        # rather than letting first boot die on the CLI's own error.
        if [ -z "${ADMIN_EMAIL:-}" ]; then
            echo "ERROR: SYSTEMPROMPT_ADMIN_EMAIL (or ADMIN_EMAIL) is required on first boot." >&2
            echo "  It identifies the platform admin on sign-in and consent screens," >&2
            echo "  so it must be an address you control. Set it in your .env or as" >&2
            echo "  an environment variable on this service, then start again." >&2
            exit 1
        fi
        # Why: setup logs in to the database it records and writes nothing when
        # it cannot, so it is handed DATABASE_URL's parts (it reads no URL) once
        # Postgres answers.
        wait_for_postgres
        db_parts="$(python3 -c 'import sys
from urllib.parse import unquote, urlsplit
u = urlsplit(sys.argv[1])
print(u.hostname or "localhost", u.port or 5432, unquote(u.username or ""), u.path.lstrip("/"), sep="\n")
print(unquote(u.password or ""))' "$DATABASE_URL")"
        db_host="$(printf '%s\n' "$db_parts" | sed -n 1p)"
        db_port="$(printf '%s\n' "$db_parts" | sed -n 2p)"
        db_user="$(printf '%s\n' "$db_parts" | sed -n 3p)"
        db_name="$(printf '%s\n' "$db_parts" | sed -n 4p)"
        printf '%s\n' "$db_parts" | sed -n 5p | /app/bin/systemprompt admin setup -e docker \
            --db-host "$db_host" --db-port "$db_port" \
            --db-user "$db_user" --db-name "$db_name" --db-password-stdin \
            --admin-email "$ADMIN_EMAIL" \
            "$@" --yes --no-migrate

        profile_created=true

        # Setup authors a localhost dev profile; patch the parts the
        # container environment dictates.
        # 1. Bind publicly (Render/compose port detection needs 0.0.0.0).
        #    Overridable via HOST for platforms whose internal networking is
        #    IPv6-only (Railway healthchecks need HOST=::).
        # Quoted: bare "::" (IPv6 any) is invalid YAML.
        sed -i "s/^  host: 127\.0\.0\.1$/  host: \"${HOST:-0.0.0.0}\"/" "$PROFILE_FILE"
        # 1b. Binaries ship in /app/bin, not a cargo target dir.
        sed -i 's|^  bin: .*|  bin: /app/bin|' "$PROFILE_FILE"
        # 2. Point at the real database, not setup's generated localhost one.
        jq --arg db "$DATABASE_URL" '.database_url = $db' "$SECRETS_FILE" \
            > "$SECRETS_FILE.tmp" && mv "$SECRETS_FILE.tmp" "$SECRETS_FILE"
        chmod 600 "$SECRETS_FILE"
        # 3. Advertise the public URL when the platform provides one
        #    (EXTERNAL_URL, or RENDER_EXTERNAL_URL via the fallback above).
        if [ -n "${EXTERNAL_URL:-}" ]; then
            sed -i "s|^  api_external_url: .*|  api_external_url: ${EXTERNAL_URL}|" "$PROFILE_FILE"
            sed -i "/^  cors_allowed_origins:/a\\  - ${EXTERNAL_URL}" "$PROFILE_FILE"
        fi
    fi
fi

python3 /app/migrate-profile.py "$PROFILE_FILE"
if [ -z "${SYSTEMPROMPT_PROFILE_DIR:-}" ]; then
    PROFILE_CREATED="$profile_created" python3 /app/container-state.py "$PROFILE_FILE"
fi

export SYSTEMPROMPT_PROFILE="$PROFILE_FILE"

if [ "$postgres_ready" = false ]; then wait_for_postgres; fi

echo "Running database migrations..."
# A managed volume/database outlives the image, so a database seeded by an older
# tag can carry checksums for migrations that were since edited in the source
# tree. --repair-drift repairs that case, and only that case, then retries
# once; every other failure aborts boot with core's classified error and hint
# (a blind repair-and-retry used to repeat the same failure twice).
/app/bin/systemprompt infra db migrate --repair-drift

echo "Ensuring bootstrap admin user..."
/app/bin/systemprompt admin bootstrap

# Why: a fresh database holds no config planes, and core reports the server
# degraded (governed requests pass ungoverned) until the declared services
# config is applied. The apply is a three-way merge that leaves conflicts for
# /admin/sync, so running it on every boot is the deploy step, not a reset.
echo "Applying declared configuration..."
/app/bin/systemprompt admin config apply --reason "container boot"

# web/dist is node-local and not shipped in the image. The scheduler's
# bootstrap run of publish_pipeline takes a database-wide advisory lock, so in
# a multi-node deployment only one node renders its site at boot and the rest
# serve 404 until a later tick lands on them. The manual runner takes no lock:
# render here, on every node, before the server accepts traffic. A failure is
# logged rather than fatal so a content problem never takes the gateway down.
if [ "${FLAVOUR_SITE:-true}" = true ]; then
    echo "Publishing web assets for this node..."
    if ! /app/bin/systemprompt infra jobs run publish_pipeline; then
        echo "WARN: publish_pipeline failed; the public site may 404 on this node until the next scheduled run." >&2
    fi
else
    echo "Publishing console assets for this node..."
    for job in bundle_admin_css copy_extension_assets; do
        if ! /app/bin/systemprompt infra jobs run "$job"; then
            echo "WARN: $job failed; the console may render unstyled on this node until it is re-run." >&2
        fi
    done
fi

echo "Starting services..."
exec /app/bin/systemprompt infra services start --foreground
