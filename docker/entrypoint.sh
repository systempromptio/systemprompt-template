#!/bin/sh
# Container entrypoint for systemprompt-template.
# Authors a profile via `systemprompt admin setup` on first boot,
# waits for Postgres, runs migrations, starts the server.
set -eu

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
else
    if [ -z "${ANTHROPIC_API_KEY:-}" ] && [ -z "${OPENAI_API_KEY:-}" ] && [ -z "${GEMINI_API_KEY:-}" ]; then
        echo "ERROR: set at least one of ANTHROPIC_API_KEY, OPENAI_API_KEY, GEMINI_API_KEY in .env" >&2
        exit 1
    fi
    if [ -z "${DATABASE_URL:-}" ]; then
        echo "ERROR: DATABASE_URL is required." >&2
        exit 1
    fi

    if [ ! -f "$PROFILE_FILE" ]; then
        echo "Generating profile via admin setup..."
        # Default provider = first configured key (setup picks up the
        # ANTHROPIC/OPENAI/GEMINI_API_KEY env vars itself).
        if [ -n "${ANTHROPIC_API_KEY:-}" ]; then DEFAULT_PROVIDER=anthropic
        elif [ -n "${OPENAI_API_KEY:-}" ]; then DEFAULT_PROVIDER=openai
        else DEFAULT_PROVIDER=gemini
        fi
        # Why: core requires --admin-email since 0.41.0. It refuses to invent
        # one because the address is shown as the operator's identity on the
        # device-link consent screen, directly above the control that mints a
        # durable personal access token. Ask here, with the variable named,
        # rather than letting first boot die on the CLI's own error.
        if [ -z "${ADMIN_EMAIL:-}" ]; then
            echo "ERROR: ADMIN_EMAIL is required on first boot." >&2
            echo "  It identifies the platform admin on sign-in and consent screens," >&2
            echo "  so it must be an address you control. Set it in your .env or as" >&2
            echo "  an environment variable on this service, then start again." >&2
            exit 1
        fi
        /app/bin/systemprompt admin setup -e docker \
            --admin-email "$ADMIN_EMAIL" \
            --default-provider "$DEFAULT_PROVIDER" --yes --no-migrate

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

export SYSTEMPROMPT_PROFILE="$PROFILE_FILE"

# Probe DATABASE_URL directly when provided (managed Postgres, e.g. Render);
# fall back to the compose-style host/user/db vars otherwise.
if [ -n "${DATABASE_URL:-}" ]; then
    pg_probe() { pg_isready -d "$DATABASE_URL"; }
    echo "Waiting for Postgres at DATABASE_URL host..."
else
    PG_HOST="${PG_HOST:-postgres}"
    PG_USER="${PG_USER:-systemprompt}"
    PG_DB="${PG_DB:-systemprompt}"
    pg_probe() { pg_isready -h "$PG_HOST" -U "$PG_USER" -d "$PG_DB"; }
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

if [ ! -f /app/signing_key.pem ]; then
    echo "Generating signing key..."
    /app/bin/systemprompt admin keys generate --output /app/signing_key.pem
fi

echo "Running database migrations..."
# A managed volume/database outlives the image, so a database seeded by an older
# tag can carry checksums for migrations that were since edited in the source
# tree. Reconcile the tracking table and retry once; anything else is a real
# migration failure and still aborts boot.
if ! /app/bin/systemprompt infra db migrate; then
    echo "Migration failed; reconciling migration checksums and retrying..." >&2
    /app/bin/systemprompt infra db migrate-repair --apply
    /app/bin/systemprompt infra db migrate
fi

echo "Ensuring bootstrap admin user..."
/app/bin/systemprompt admin bootstrap

echo "Starting services..."
exec /app/bin/systemprompt infra services start --foreground
