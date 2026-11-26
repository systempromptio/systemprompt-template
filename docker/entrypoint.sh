#!/bin/sh
# Container entrypoint for systemprompt-template.
# Authors a profile via `systemprompt admin setup` on first boot, prepares
# durable state, waits for Postgres, runs migrations, starts the server. The
# image's flavour (Dockerfile ARGs) decides whether a provider key is required
# (FLAVOUR_REQUIRES_AI_KEY) and whether the site or only the console assets
# are published (FLAVOUR_SITE). The runtime is distroless plus busybox, jq and
# lsof (docker/runtime-tools.sh): everything here is POSIX sh over those.
set -eu
umask 077
profile_created=false

# Railway mounts volumes as root. Prepare ownership, then run the gateway
# under the same unprivileged account used on other container hosts.
if [ "$(id -u)" = 0 ] && [ -n "${SYSTEMPROMPT_DATA_DIR:-}" ]; then
    mkdir -p "$SYSTEMPROMPT_DATA_DIR"
    chown -R app:app "$SYSTEMPROMPT_DATA_DIR"
    export HOME=/home/app
    exec setpriv --reuid=app --regid=app --init-groups "$0" "$@"
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

# A platform volume at SYSTEMPROMPT_DATA_DIR holds every durable path; each is
# moved there once and the image path becomes a link to it, so packaged web
# assets elsewhere under /app are never masked by the volume.
attach_state() {
    for relative in .systemprompt storage/data storage/files/uploads \
        storage/files/images/generated storage/files/audio storage/files/video \
        storage/files/documents; do
        source="/app/$relative"
        destination="$SYSTEMPROMPT_DATA_DIR/$relative"
        mkdir -p "$destination"
        if [ -L "$source" ]; then
            if [ "$(readlink -f "$source")" != "$(readlink -f "$destination")" ]; then
                echo "ERROR: $source points outside the configured state directory" >&2
                exit 1
            fi
            continue
        fi
        if [ -e "$source" ]; then
            (cd "$source" && find . -exec sh -c '
                for item; do
                    if [ -d "$item" ]; then mkdir -p "$0/$item"
                    elif [ ! -e "$0/$item" ]; then cp -a "$item" "$0/$item"; fi
                done' "$destination" {} +)
            rm -rf "$source"
        fi
        mkdir -p "$(dirname "$source")"
        ln -s "$destination" "$source"
    done
}

if [ -n "${SYSTEMPROMPT_DATA_DIR:-}" ]; then
    attach_state
fi

percent_decode() {
    printf '%b' "$(printf '%s' "$1" | sed 's/\\/\\\\/g; s/%\([0-9A-Fa-f][0-9A-Fa-f]\)/\\x\1/g')"
}

# Splits a postgres:// URL into db_host, db_port, db_user, db_password, db_name.
parse_database_url() {
    rest="${1#*://}"
    rest="${rest%%\?*}"
    authority="${rest%%/*}"
    db_name=""
    case "$rest" in */*) db_name="$(percent_decode "${rest#*/}")" ;; esac
    userinfo=""
    hostport="$authority"
    case "$authority" in *@*) userinfo="${authority%@*}"; hostport="${authority##*@}" ;; esac
    db_user="$(percent_decode "${userinfo%%:*}")"
    db_password=""
    case "$userinfo" in *:*) db_password="$(percent_decode "${userinfo#*:}")" ;; esac
    db_port=""
    case "$hostport" in
        \[*\]*) db_host="${hostport%%]*}"; db_host="${db_host#[}"; db_port="${hostport##*]}"; db_port="${db_port#:}" ;;
        *:*) db_host="${hostport%:*}"; db_port="${hostport##*:}" ;;
        *) db_host="$hostport" ;;
    esac
    db_host="${db_host:-localhost}"
    db_port="${db_port:-5432}"
}

be32() {
    printf "\\$(printf %03o $(($1 >> 24 & 255)))\\$(printf %03o $(($1 >> 16 & 255)))\\$(printf %03o $(($1 >> 8 & 255)))\\$(printf %03o $(($1 & 255)))"
}

# pg_isready's answer for one attempt: a startup packet is sent and the server
# is accepting connections unless it refuses the socket or reports it is
# starting up (SQLSTATE 57P03); an authentication request or any other error
# means it is up, as PQping reads it.
postgres_ready() {
    parse_database_url "$1"
    probe_user="${db_user:-postgres}"
    probe_db="${db_name:-$probe_user}"
    length=$((25 + ${#probe_user} + ${#probe_db}))
    reply="$({ be32 "$length"; be32 196608; printf 'user\000%s\000database\000%s\000\000' "$probe_user" "$probe_db"; sleep 2; } \
        | timeout 3 nc "$db_host" "$db_port" 2>/dev/null | head -c 512 | tr '\000' ' ')" || true
    case "$reply" in
        "") return 1 ;;
        E*57P03*) return 1 ;;
        *) return 0 ;;
    esac
}

postgres_ready=false
wait_for_postgres() {
    # Probe DATABASE_URL directly when provided (managed Postgres, e.g. Render);
    # fall back to the compose-style host/user/db vars otherwise.
    if [ -n "${DATABASE_URL:-}" ]; then
        pg_probe() { postgres_ready "$DATABASE_URL"; }
        echo "Waiting for Postgres at DATABASE_URL host..."
    else
        PG_HOST="${PG_HOST:-postgres}"
        PG_USER="${PG_USER:-systemprompt}"
        PG_DB="${PG_DB:-systemprompt}"
        pg_probe() { postgres_ready "postgres://$PG_USER@$PG_HOST:5432/$PG_DB"; }
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
        key_file="$(sed -n 's/^  signing_key_path: *//p' "$PROFILE_FILE" | tr -d "\"'")"
        key_file="${key_file:-/app/signing_key.pem}"
        case "$key_file" in /*) ;; *) key_file="$PROFILE_DIR/$key_file" ;; esac
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
        parse_database_url "$DATABASE_URL"
        printf '%s\n' "$db_password" | /app/bin/systemprompt admin setup -e docker \
            --db-host "$db_host" --db-port "$db_port" \
            --db-user "$db_user" --db-name "$db_name" --db-password-stdin \
            --admin-email "$ADMIN_EMAIL" \
            "$@" --yes --no-migrate

        profile_created=true

        # Setup authors a localhost dev profile; patch the parts the
        # container environment dictates.
        # Binaries ship in /app/bin, not a cargo target dir; no `admin config`
        # command sets paths.bin.
        sed -i 's|^  bin: .*|  bin: /app/bin|' "$PROFILE_FILE"
    fi
fi

export SYSTEMPROMPT_PROFILE="$PROFILE_FILE"

secrets_set() {
    jq --arg key "$1" --arg value "$2" '.[$key] = $value' "$SECRETS_FILE" > "$SECRETS_FILE.tmp"
    mv "$SECRETS_FILE.tmp" "$SECRETS_FILE"
    chmod 600 "$SECRETS_FILE"
}

# The container environment owns these on every boot; the rest of the profile
# is what admin setup authored plus the flavour's base layer
# (<profiles>/base.yaml, which core merges under profile.yaml).
configure_profile() {
    cp /app/profile.base.yaml "$(dirname "$PROFILE_DIR")/base.yaml"
    case "$(printf '%s' "${ALLOW_REGISTRATION:-false}" | tr '[:upper:]' '[:lower:]')" in
        true) allow_registration=true ;;
        *) allow_registration=false ;;
    esac
    # Why: no `admin config` command sets security.allow_registration.
    sed -i "s/^  allow_registration: .*/  allow_registration: $allow_registration/" "$PROFILE_FILE"
    set -- --host "${HOST:-0.0.0.0}" --port "${PORT:-8080}"
    if [ -n "$EXTERNAL_URL" ]; then set -- "$@" --external-url "${EXTERNAL_URL%/}"; fi
    /app/bin/systemprompt admin config server set "$@" > /dev/null
    /app/bin/systemprompt admin config rate-limits enable > /dev/null
    secrets_set database_url "$DATABASE_URL"
    # Optional read/write split: reads stay on DATABASE_URL, writes go to the
    # primary named here (core refuses a standby as the write target).
    if [ -n "${DATABASE_WRITE_URL:-}" ]; then secrets_set database_write_url "$DATABASE_WRITE_URL"; fi
}

# Platform-generated identity inputs (one-click templates) apply to the profile
# this boot created; a redeploy keeps the identity it already has.
apply_platform_identity() {
    if [ -n "${OAUTH_AT_REST_PEPPER:-}" ]; then
        if [ "${#OAUTH_AT_REST_PEPPER}" -lt 32 ]; then
            echo "ERROR: OAUTH_AT_REST_PEPPER must contain at least 32 characters" >&2
            exit 1
        fi
        secrets_set oauth_at_rest_pepper "$OAUTH_AT_REST_PEPPER"
    fi
    if [ -n "${MANIFEST_SIGNING_SECRET_SEED:-}" ]; then
        if [ "$(printf '%s' "$MANIFEST_SIGNING_SECRET_SEED" | base64 -d 2>/dev/null | wc -c)" -ne 32 ]; then
            echo "ERROR: MANIFEST_SIGNING_SECRET_SEED must encode 32 bytes" >&2
            exit 1
        fi
        secrets_set manifest_signing_secret_seed "$MANIFEST_SIGNING_SECRET_SEED"
    fi
}

if [ -z "${SYSTEMPROMPT_PROFILE_DIR:-}" ]; then
    if [ "$profile_created" = true ]; then apply_platform_identity; fi
    configure_profile
fi

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
    # Why: one invocation boots the CLI kernel once for both jobs.
    if ! /app/bin/systemprompt infra jobs run bundle_admin_css copy_extension_assets --sequential; then
        echo "WARN: publishing console assets failed; the console may render unstyled on this node until bundle_admin_css and copy_extension_assets are re-run." >&2
    fi
fi

echo "Starting services..."
exec /app/bin/systemprompt infra services start --foreground
