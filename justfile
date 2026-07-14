# systemprompt-template
set dotenv-load

CLI_RELEASE := "target/release/systemprompt"

# Use newest binary (release vs debug, whichever is most recent)
CLI := if path_exists("target/release/systemprompt") == "true" { \
    if path_exists("target/debug/systemprompt") == "true" { \
        `[ target/release/systemprompt -nt target/debug/systemprompt ] && echo target/release/systemprompt || echo target/debug/systemprompt` \
    } else { \
        "target/release/systemprompt" \
    } \
} else if path_exists("target/debug/systemprompt") == "true" { \
    "target/debug/systemprompt" \
} else { \
    "echo 'ERROR: No CLI binary found. Run: just build' && exit 1" \
}

# Default: run CLI with any arguments
default *ARGS:
    {{CLI}} {{ARGS}}

# Run CLI with full session context (profile + auth token)
cli *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    SESSION_FILE="{{justfile_directory()}}/.systemprompt/sessions/index.json"
    if [ -f "$SESSION_FILE" ]; then
        ACTIVE_KEY=$(jq -r '.active_key // "local"' "$SESSION_FILE")
        export SYSTEMPROMPT_PROFILE=$(jq -r ".sessions[\"$ACTIVE_KEY\"].profile_path // empty" "$SESSION_FILE")
        export SYSTEMPROMPT_AUTH_TOKEN=$(jq -r ".sessions[\"$ACTIVE_KEY\"].session_token // empty" "$SESSION_FILE")
    fi
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ]; then
        export SYSTEMPROMPT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"
    fi
    exec {{CLI}} {{ARGS}}

# Get DATABASE_URL from profile secrets (for sqlx compile-time checks)
_db-url:
    @if [ -n "$SYSTEMPROMPT_PROFILE" ] && [ -f "$SYSTEMPROMPT_PROFILE" ]; then \
        PROFILE_DIR="$(dirname "$SYSTEMPROMPT_PROFILE")"; \
        SECRETS_PATH="$(yq -r '.secrets.secrets_path // "./secrets.json"' "$SYSTEMPROMPT_PROFILE")"; \
        if [ "${SECRETS_PATH#/}" = "$SECRETS_PATH" ]; then \
            SECRETS_FILE="$PROFILE_DIR/$SECRETS_PATH"; \
        else \
            SECRETS_FILE="$SECRETS_PATH"; \
        fi; \
        if [ -f "$SECRETS_FILE" ]; then \
            jq -r '.database_url' "$SECRETS_FILE"; \
        else \
            echo "postgres://systemprompt:systemprompt@localhost:5432/systemprompt"; \
        fi; \
    else \
        cat .systemprompt/tenants.json 2>/dev/null | jq -r '.tenants[] | select(.tenant_type == "local") | .database_url' | head -1 || echo "postgres://systemprompt:systemprompt@localhost:5432/systemprompt"; \
    fi

# ══════════════════════════════════════════════════════════════════════════════
# BUILD & CHECK
# ══════════════════════════════════════════════════════════════════════════════

# Build (Windows) - always uses offline mode
[windows]
build *FLAGS:
    $env:SQLX_OFFLINE="true"; cargo build --workspace {{FLAGS}}

# Build (Unix) - tries database, falls back to offline
[unix]
build *FLAGS:
    #!/usr/bin/env bash
    set -euo pipefail
    # Default to the `local` profile when one is set up but no SYSTEMPROMPT_PROFILE
    # is explicitly exported — keeps the in-build migrate step from failing
    # with "Profile '' not found" on a fresh clone where setup-local writes
    # secrets.json before invoking `just build`.
    SECRETS_FILE_DEFAULT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ] && [ -f "$SECRETS_FILE_DEFAULT_PROFILE" ]; then
        export SYSTEMPROMPT_PROFILE="local"
    else
        export SYSTEMPROMPT_PROFILE="${SYSTEMPROMPT_PROFILE:-}"
    fi
    # aws-lc-sys refuses to build with GCC <10 due to bug #95189.
    # Force clang if available so release (LTO) builds succeed.
    if command -v clang >/dev/null 2>&1; then
        export CC="${CC:-clang}"
        export CXX="${CXX:-clang++}"
    fi
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    USE_OFFLINE=false
    db_reachable() {
        local url="$1"
        local pgcmd=""
        if command -v pg_isready >/dev/null 2>&1; then pgcmd="pg_isready"
        elif [ -x /opt/homebrew/opt/libpq/bin/pg_isready ]; then pgcmd="/opt/homebrew/opt/libpq/bin/pg_isready"
        elif [ -x /usr/local/opt/libpq/bin/pg_isready ]; then pgcmd="/usr/local/opt/libpq/bin/pg_isready"
        fi
        if [ -n "$pgcmd" ]; then
            "$pgcmd" -d "$url" -t 2 >/dev/null 2>&1 && return 0 || return 1
        fi
        local hostport="${url#*@}"; hostport="${hostport%%/*}"
        local host="${hostport%:*}"; local port="${hostport##*:}"
        [ "$port" = "$host" ] && port=5432
        (exec 3<>/dev/tcp/"$host"/"$port") >/dev/null 2>&1 && { exec 3<&-; exec 3>&-; return 0; } || return 1
    }
    if [ -f "$SECRETS_FILE" ]; then
        DB_URL=$(sed -n 's/.*"database_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$SECRETS_FILE" 2>/dev/null | head -1)
        if [ -n "$DB_URL" ] && [ "$DB_URL" != "null" ]; then
            if db_reachable "$DB_URL"; then
                export DATABASE_URL="$DB_URL"
                echo "Using database: $DB_URL"
            else
                echo "Database not reachable, using offline mode"
                USE_OFFLINE=true
            fi
        else
            echo "No database_url in secrets, using offline mode"
            USE_OFFLINE=true
        fi
    else
        echo "No local profile secrets found, using offline mode"
        USE_OFFLINE=true
    fi
    # Sync DATABASE_URL to MCP extension directories for sqlx compile-time checks
    if [ "$USE_OFFLINE" = "false" ]; then
        for dir in extensions/mcp/*/; do
            if [ -f "$dir/Cargo.toml" ]; then
                echo "DATABASE_URL=$DATABASE_URL" > "$dir/.env"
            fi
        done
    fi
    cargo update systemprompt --quiet 2>/dev/null || true
    if [ "$USE_OFFLINE" = "true" ]; then
        SQLX_OFFLINE=true cargo build --workspace {{FLAGS}}
    else
        # Apply pending schema migrations before the online sqlx compile-time
        # check sees the live DB. Build the CLI in offline mode first so
        # drift between checked-in `.sqlx/` and the unmigrated live schema
        # can't deadlock the bootstrap.
        echo "Applying pending migrations before online build..."
        SQLX_OFFLINE=true cargo build --bin systemprompt --quiet
        target/debug/systemprompt infra db migrate
        SQLX_OFFLINE=false cargo build --workspace {{FLAGS}}
    fi

# Clippy (Windows) - always uses offline mode
[windows]
clippy *FLAGS: lint-no-synthesis lint-gates
    $env:SQLX_OFFLINE="true"; cargo clippy --workspace {{FLAGS}} -- -D warnings

# Clippy (Unix) - tries database, falls back to offline
[unix]
clippy *FLAGS: lint-no-synthesis lint-gates
    #!/usr/bin/env bash
    set -euo pipefail
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    USE_OFFLINE=false
    db_reachable() {
        local url="$1"
        local pgcmd=""
        if command -v pg_isready >/dev/null 2>&1; then pgcmd="pg_isready"
        elif [ -x /opt/homebrew/opt/libpq/bin/pg_isready ]; then pgcmd="/opt/homebrew/opt/libpq/bin/pg_isready"
        elif [ -x /usr/local/opt/libpq/bin/pg_isready ]; then pgcmd="/usr/local/opt/libpq/bin/pg_isready"
        fi
        if [ -n "$pgcmd" ]; then
            "$pgcmd" -d "$url" -t 2 >/dev/null 2>&1 && return 0 || return 1
        fi
        local hostport="${url#*@}"; hostport="${hostport%%/*}"
        local host="${hostport%:*}"; local port="${hostport##*:}"
        [ "$port" = "$host" ] && port=5432
        (exec 3<>/dev/tcp/"$host"/"$port") >/dev/null 2>&1 && { exec 3<&-; exec 3>&-; return 0; } || return 1
    }
    if [ -f "$SECRETS_FILE" ]; then
        DB_URL=$(sed -n 's/.*"database_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$SECRETS_FILE" 2>/dev/null | head -1)
        if [ -n "$DB_URL" ] && [ "$DB_URL" != "null" ]; then
            if db_reachable "$DB_URL"; then
                export DATABASE_URL="$DB_URL"
            else
                USE_OFFLINE=true
            fi
        else
            USE_OFFLINE=true
        fi
    else
        USE_OFFLINE=true
    fi
    if [ "$USE_OFFLINE" = "true" ]; then
        SQLX_OFFLINE=true cargo clippy --workspace {{FLAGS}} -- -D warnings
    else
        SQLX_OFFLINE=false cargo clippy --workspace {{FLAGS}} -- -D warnings
    fi

# Source gates ported from systemprompt-core (scripts/*.sh)
lint-gates:
    #!/usr/bin/env bash
    set -euo pipefail
    bash scripts/lint-schema.sh
    bash scripts/lint-extensions.sh
    bash scripts/check-sqlx.sh
    bash scripts/check-http-errors.sh
    bash scripts/check-test-value.sh

# Structural guard: no string-literal `UserId::new("...")` in extension code.
# String literals are how principal synthesis sneaks in — every legitimate
# UserId::new call takes a validated identifier as a variable, never a literal.
# Allowlisted: test code (regression tests intentionally construct ids) and
# any future bootstrap/provisioning module.
lint-no-synthesis:
    #!/usr/bin/env bash
    set -euo pipefail
    hits=$(grep -rEn 'UserId::new\("' extensions/ \
        --include='*.rs' \
        --exclude-dir=tests \
        --exclude-dir=bootstrap \
        || true)
    if [ -n "$hits" ]; then
        echo "error: forbidden synthesized principal — UserId::new with string literal"
        echo "$hits"
        echo
        echo "UserId::new must take a validated identifier (from cookie, query,"
        echo "JWT claim, or DB row), never a hard-coded literal. If this is"
        echo "legitimate bootstrap code, move it to extensions/**/bootstrap/."
        exit 1
    fi

# Prepare SQLx offline query cache (requires running database)
prepare:
    #!/usr/bin/env bash
    set -euo pipefail
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    if [ ! -f "$SECRETS_FILE" ]; then
        echo "Error: No local profile secrets found at $SECRETS_FILE"
        echo "Run 'just db-up' first to start the database"
        exit 1
    fi
    DB_URL=$(jq -r '.database_url // empty' "$SECRETS_FILE" 2>/dev/null)
    if [ -z "$DB_URL" ] || [ "$DB_URL" = "null" ]; then
        echo "Error: No database_url in secrets"
        exit 1
    fi
    PG_ISREADY=""
    if command -v pg_isready >/dev/null 2>&1; then PG_ISREADY="pg_isready"
    elif [ -x /opt/homebrew/opt/libpq/bin/pg_isready ]; then PG_ISREADY="/opt/homebrew/opt/libpq/bin/pg_isready"
    elif [ -x /usr/local/opt/libpq/bin/pg_isready ]; then PG_ISREADY="/usr/local/opt/libpq/bin/pg_isready"
    fi
    if [ -z "$PG_ISREADY" ] || ! "$PG_ISREADY" -d "$DB_URL" -t 2 >/dev/null 2>&1; then
        echo "Error: Database not reachable at $DB_URL"
        echo "Run 'just db-up' first to start the database"
        exit 1
    fi
    # Apply pending migrations before sqlx prepare — otherwise the macros
    # see a schema older than the code references and fail with
    # "relation ... does not exist". Skipped if no CLI binary exists yet
    # (first-time bootstrap before any build).
    if [ -x "{{CLI}}" ]; then
        echo "Applying pending migrations..."
        {{CLI}} infra db migrate
    else
        echo "Warning: no systemprompt binary yet; skipping migrate step."
        echo "  If sqlx prepare fails with 'relation does not exist',"
        echo "  build first ('just build') then re-run 'just prepare'."
    fi
    echo "Preparing SQLx offline cache..."
    export DATABASE_URL="$DB_URL"
    # Drop any stale incremental sqlx artifacts so the query macros re-run
    # against the freshly-migrated schema. Without this, target/ may cache
    # check results from before the migrations were applied.
    cargo clean -p systemprompt-database 2>/dev/null || true
    # Workspace-level prepare (catches lib crates)
    cargo sqlx prepare --workspace
    # Per-crate prepare for binary/extension crates that cargo sqlx skips
    EXTENSION_DIRS="extensions/cli/activity extensions/cli/slack extensions/web extensions/marketplace extensions/mcp/systemprompt"
    for dir in $EXTENSION_DIRS; do
        if [ -f "{{justfile_directory()}}/$dir/Cargo.toml" ]; then
            echo "  Preparing $dir..."
            (cd "{{justfile_directory()}}/$dir" && cargo sqlx prepare 2>&1 | tail -1) || true
            if ls "{{justfile_directory()}}/$dir/.sqlx/"*.json >/dev/null 2>&1; then
                cp "{{justfile_directory()}}/$dir/.sqlx/"*.json "{{justfile_directory()}}/.sqlx/"
            fi
        fi
    done
    echo "SQLx cache prepared successfully ($(ls {{justfile_directory()}}/.sqlx/ | wc -l) queries cached)"

# ══════════════════════════════════════════════════════════════════════════════
# SERVICES & DATABASE
# ══════════════════════════════════════════════════════════════════════════════

# Start server (always uses local profile)
start:
    {{CLI}} infra services start --profile local

# Start server with release binary
start-release:
    {{CLI_RELEASE}} infra services start --profile local

# Run migrations
migrate:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ]; then
        export SYSTEMPROMPT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"
    fi
    {{CLI}} infra db migrate

# When an already-applied migration file is edited (e.g. a seed fix), its
# stored checksum stops matching the file and `migrate` / `start` refuse to
# proceed. `infra db migrate-repair` re-aligns the tracking table by dropping
# the drifted rows and re-applying those migrations — every migration is
# idempotent (guarded seeds or CREATE ... IF NOT EXISTS), so re-running them
# re-records the current checksum without touching your data.
# Repair migration checksum drift in place — no data loss, no destructive reset.
repair-migrations:
    {{CLI}} infra db migrate-repair --apply

# Per-clone docker compose project name. Derived from the absolute justfile directory
# so a second clone on the same host gets its own containers and volumes.
_project_name TENANT:
    #!/usr/bin/env bash
    set -euo pipefail
    HASH=$(printf '%s' "{{justfile_directory()}}" | { sha256sum 2>/dev/null || shasum -a 256; } | cut -c1-8)
    LEAF=$(basename "{{justfile_directory()}}" | tr '_' '-' | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9-]/-/g')
    printf 'sp-%s-%s-%s\n' "$LEAF" "$HASH" "{{TENANT}}"

# Start PostgreSQL for a specific tenant (default: local)
db-up TENANT="local":
    docker compose -p "$(just _project_name {{TENANT}})" -f .systemprompt/docker/{{TENANT}}.yaml up -d

# Stop PostgreSQL for a specific tenant
db-down TENANT="local":
    docker compose -p "$(just _project_name {{TENANT}})" -f .systemprompt/docker/{{TENANT}}.yaml down

# Show PostgreSQL logs for a specific tenant
db-logs TENANT="local":
    docker compose -p "$(just _project_name {{TENANT}})" -f .systemprompt/docker/{{TENANT}}.yaml logs -f

# List all tenant databases
db-list:
    @ls -1 .systemprompt/docker/*.yaml 2>/dev/null | xargs -I {} basename {} .yaml || echo "No tenant databases found"

# ══════════════════════════════════════════════════════════════════════════════
# AUTH & TENANT & PROFILE
# ══════════════════════════════════════════════════════════════════════════════

# Authenticate with SystemPrompt Cloud
login ENV="production":
    {{CLI}} cloud auth login {{ENV}}

# Clear saved credentials
logout:
    {{CLI}} cloud auth logout

# Show current user and tenant
whoami:
    {{CLI}} cloud auth whoami

# Tenant operations (interactive menu)
tenant:
    {{CLI}} cloud tenant

# Set up a local-only profile + Docker Postgres (no cloud, no login required).
# Pass keys as positional args, or leave blank to be prompted interactively:
#   just setup-local sk-ant-... sk-... AIza...
# Port and Postgres port can be overridden for running multiple clones on one host:
#   just setup-local sk-ant-... "" "" 8081 5433
setup-local ANTHROPIC_KEY="" OPENAI_KEY="" GEMINI_KEY="" HTTP_PORT="8080" PG_PORT="5432":
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="{{justfile_directory()}}"
    PROFILE_DIR="$ROOT/.systemprompt/profiles/local"
    DOCKER_DIR="$ROOT/.systemprompt/docker"
    ANTHROPIC_KEY="{{ANTHROPIC_KEY}}"
    OPENAI_KEY="{{OPENAI_KEY}}"
    GEMINI_KEY="{{GEMINI_KEY}}"
    HTTP_PORT="{{HTTP_PORT}}"
    PG_PORT="{{PG_PORT}}"
    export SYSTEMPROMPT_PROFILE="$PROFILE_DIR/profile.yaml"
    # Whether a key was passed as a positional arg. When none is and there is
    # nothing to preserve, generation still needs a provider: on a TTY we let
    # `admin setup` drive its own "Select your AI provider" menu (the CLI owns
    # the prompt); off a TTY we cannot prompt, so keys must come as args. A
    # developer who keeps .systemprompt/ across reclones re-runs with no args
    # and is never asked again (the profile.yaml guard below skips generation).
    HAS_KEY=false
    if [ -n "$ANTHROPIC_KEY" ] || [ -n "$OPENAI_KEY" ] || [ -n "$GEMINI_KEY" ]; then
        HAS_KEY=true
    fi
    if [ "$HAS_KEY" = false ] && [ ! -f "$PROFILE_DIR/secrets.json" ] && [ ! -t 0 ]; then
        echo ""
        echo "================================================================"
        echo "  setup-local needs an AI provider API key"
        echo "================================================================"
        echo ""
        echo "  Not running on a TTY, so the provider menu can't be shown."
        echo "  Pass a key as an argument (one of Anthropic, OpenAI, Gemini):"
        echo "    just setup-local <anthropic_key> [openai_key] [gemini_key]"
        echo ""
        exit 1
    fi
    if [ ! -x target/debug/systemprompt ] && [ ! -x target/release/systemprompt ]; then
        echo "Building debug binary..."
        just build
    fi
    # Resolve the binary at runtime: the {{CLI}} variable is evaluated by `just`
    # at parse time, so on a cold clone (no binary yet) it expands to an error
    # stub — useless for the bootstrap/keygen calls below, which run only after
    # the build above has produced the binary.
    if [ -x target/release/systemprompt ]; then
        BIN="$ROOT/target/release/systemprompt"
    else
        BIN="$ROOT/target/debug/systemprompt"
    fi
    mkdir -p "$PROFILE_DIR" "$DOCKER_DIR"
    if [ ! -f "$DOCKER_DIR/local.yaml" ]; then
        echo "Writing Docker compose for local Postgres (host port $PG_PORT)..."
        cat > "$DOCKER_DIR/local.yaml" <<YAML
    services:
      postgres:
        image: postgres:18-alpine
        restart: unless-stopped
        environment:
          POSTGRES_USER: systemprompt
          POSTGRES_PASSWORD: 123
          POSTGRES_DB: systemprompt
        ports:
          - "${PG_PORT}:5432"
        volumes:
          - postgres_data:/var/lib/postgresql
        healthcheck:
          test: ["CMD-SHELL", "pg_isready -U systemprompt -d systemprompt"]
          interval: 5s
          timeout: 5s
          retries: 5
    volumes:
      postgres_data: {}
    YAML
    fi
    if [ ! -f "$PROFILE_DIR/profile.yaml" ]; then
        echo "Generating profile + provider registry + secrets via 'admin setup'..."
        if [ "$HAS_KEY" = true ]; then
            # Keys supplied as args: fully non-interactive. The default provider
            # is the first key given, so the generated config (the providers
            # registry, gateway default, ai/config.yaml) is consistent with the
            # single key.
            KEY_ARGS=()
            DEFAULT_PROVIDER=""
            if [ -n "$ANTHROPIC_KEY" ]; then KEY_ARGS+=(--anthropic-key "$ANTHROPIC_KEY"); [ -z "$DEFAULT_PROVIDER" ] && DEFAULT_PROVIDER=anthropic; fi
            if [ -n "$OPENAI_KEY" ]; then KEY_ARGS+=(--openai-key "$OPENAI_KEY"); [ -z "$DEFAULT_PROVIDER" ] && DEFAULT_PROVIDER=openai; fi
            if [ -n "$GEMINI_KEY" ]; then KEY_ARGS+=(--gemini-key "$GEMINI_KEY"); [ -z "$DEFAULT_PROVIDER" ] && DEFAULT_PROVIDER=gemini; fi
            "$BIN" admin setup --yes --no-migrate --environment local \
                --db-host localhost --db-port "$PG_PORT" \
                --db-user systemprompt --db-password 123 --db-name systemprompt \
                --default-provider "$DEFAULT_PROVIDER" \
                "${KEY_ARGS[@]}"
        else
            # No key arg: let the CLI prompt for which provider to use. DB,
            # environment, and migrations stay non-interactive (flags + env);
            # only the provider selection is interactive, and the chosen
            # provider becomes the default.
            SYSTEMPROMPT_NON_INTERACTIVE=1 "$BIN" admin setup --no-migrate --environment local \
                --db-host localhost --db-port "$PG_PORT" \
                --db-user systemprompt --db-password 123 --db-name systemprompt
        fi
        if [ "$HTTP_PORT" != "8080" ]; then
            "$BIN" admin config server set --port "$HTTP_PORT" \
                --api-server-url "http://localhost:${HTTP_PORT}" \
                --api-internal-url "http://localhost:${HTTP_PORT}" \
                --api-external-url "http://localhost:${HTTP_PORT}"
            # The authz hook URL is an absolute webhook target baked at
            # `admin setup` time on the default port; re-point it at the
            # chosen port so the gateway's govern callback reaches this server.
            "$BIN" admin config governance set --mode webhook \
                --url "http://localhost:${HTTP_PORT}/api/public/govern/authz"
        fi
    elif [ "$HAS_KEY" = true ]; then
        # Profile generation is one-shot, guarded on profile.yaml. `just db-down`
        # drops the database but leaves the profile, so a re-run with different
        # keys would silently keep the old provider registry. Say so loudly and
        # point at the one command that actually re-provisions.
        echo ""
        echo "================================================================"
        echo "  Existing profile reused — supplied keys were NOT applied"
        echo "================================================================"
        echo ""
        echo "  $PROFILE_DIR/profile.yaml already exists, so 'admin setup' was"
        echo "  skipped and the provider registry/keys are unchanged."
        echo "  To re-provision from the keys you just passed:"
        echo ""
        echo "    rm -rf \"$PROFILE_DIR\" && just setup-local <keys...> $HTTP_PORT $PG_PORT"
        echo ""
    fi
    mkdir -p "$ROOT/web/dist"
    echo "Building binaries (release, full workspace)..."
    just build --release
    echo "Starting local Postgres via Docker..."
    just db-up local
    echo "Waiting for Postgres to accept connections on localhost:${PG_PORT}..."
    for i in $(seq 1 60); do
        if (exec 3<>/dev/tcp/127.0.0.1/${PG_PORT}) 2>/dev/null; then
            exec 3<&- 3>&-
            # Also confirm the server actually answers pg_isready, not just a half-open socket.
            CONTAINER=$(docker compose -p "$(just _project_name local)" -f .systemprompt/docker/local.yaml ps -q postgres)
            if [ -n "$CONTAINER" ] && docker exec "$CONTAINER" pg_isready -U systemprompt -d systemprompt >/dev/null 2>&1; then
                echo "Postgres is ready."
                break
            fi
        fi
        if [ "$i" = "60" ]; then
            echo "ERROR: Postgres did not become ready within 60s." >&2
            exit 1
        fi
        sleep 1
    done
    echo "Running database migrations..."
    just migrate
    echo "Ensuring bootstrap admin user..."
    "$BIN" admin bootstrap
    if [ ! -f "$ROOT/signing_key.pem" ]; then
        echo "Generating JWT signing key..."
        "$BIN" admin keys generate --output "$ROOT/signing_key.pem"
    fi
    echo "Publishing assets..."
    just publish
    echo ""
    echo "Local setup complete. Run: just start"

# List all tenants
tenants:
    {{CLI}} cloud tenant list

# Profile operations (interactive menu)
profile:
    {{CLI}} cloud profile

# List all profiles
profiles:
    {{CLI}} cloud profile list

# ══════════════════════════════════════════════════════════════════════════════
# SYNC
# ══════════════════════════════════════════════════════════════════════════════

# Sync content to database
sync-content *ARGS:
    {{CLI}} cloud sync local content {{ARGS}}

# Sync skills to database
sync-skills *ARGS:
    {{CLI}} cloud sync local skills {{ARGS}}

# Sync all local content
sync-local:
    {{CLI}} cloud sync local content
    {{CLI}} cloud sync local skills

# Push to cloud
sync-push *ARGS:
    {{CLI}} cloud sync push {{ARGS}}

# Pull from cloud
sync-pull *ARGS:
    {{CLI}} cloud sync pull {{ARGS}}

# ══════════════════════════════════════════════════════════════════════════════
# DEPLOY
# ══════════════════════════════════════════════════════════════════════════════

# Deploy to cloud
# Note: publish_pipeline runs automatically on server startup with correct profile URLs
deploy *FLAGS:
    just build --release
    {{CLI_RELEASE}} cloud deploy {{FLAGS}}

# Check deployment status
status:
    {{CLI}} cloud status

# ══════════════════════════════════════════════════════════════════════════════
# MCP & BUILD ALL
# ══════════════════════════════════════════════════════════════════════════════

# Build all MCP servers (reads from manifest.yaml files)
build-mcp:
    DATABASE_URL="$(just _db-url)" {{CLI}} build mcp --release

# Build everything for deployment (Rust binary + MCP servers + web assets)
build-all:
    just build --release
    just build-mcp
    just web-build
    {{CLI_RELEASE}} infra jobs run publish_pipeline
    @echo "All components built"

# ══════════════════════════════════════════════════════════════════════════════
# WEB ASSETS & PUBLISHING
# ══════════════════════════════════════════════════════════════════════════════

# Copy web assets to dist (CSS, JS, images)
web-assets:
    {{CLI}} infra jobs run copy_extension_assets

# Publish: compile templates, bundle CSS/JS, copy assets, prerender content
publish:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ]; then
        export SYSTEMPROMPT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"
    fi
    {{CLI}} infra jobs run publish_pipeline

# Build web assets only (templates + CSS + JS + copy to dist)
web-build:
    {{CLI}} infra jobs run bundle_admin_css
    {{CLI}} infra jobs run copy_extension_assets

# ══════════════════════════════════════════════════════════════════════════════
# DOCKER
# ══════════════════════════════════════════════════════════════════════════════

# Build Docker image for local testing
docker-build TAG="local":
    docker build -f .systemprompt/Dockerfile -t systemprompt-template:{{TAG}} .

# Run image locally for testing
docker-run TAG="local":
    docker run -p 8080:8080 --env-file .env systemprompt-template:{{TAG}}

# Test build without pushing
docker-test:
    just build-all
    just docker-build test
    @echo "Docker build successful! Image: systemprompt-template:test"

# ══════════════════════════════════════════════════════════════════════════════
# AIR-GAPPED SCENARIO
# ══════════════════════════════════════════════════════════════════════════════

# Bring up the network-isolated air-gap stack (postgres + mock-inference + app + monitor + ingress)
airgap-up:
    #!/usr/bin/env bash
    set -euo pipefail
    # Dockerfile.airgap-prebuilt COPYs the host-built binaries from
    # deploy/scenarios/airgap/.bin/ — `target` is a symlink to a shared cargo
    # cache that buildkit can't follow, so we dereference-copy them in first
    # (mirrors scaled-up).
    if [[ ! -x target/release/systemprompt || ! -x target/release/systemprompt-mcp-agent ]]; then
        echo "ERROR: release binaries missing. Run: just build --release" >&2
        exit 1
    fi
    mkdir -p deploy/scenarios/airgap/.bin
    cp -L target/release/systemprompt           deploy/scenarios/airgap/.bin/systemprompt
    cp -L target/release/systemprompt-mcp-agent deploy/scenarios/airgap/.bin/systemprompt-mcp-agent
    docker compose -f deploy/scenarios/airgap/docker-compose.airgap.yml up -d --build

# Tear down the air-gap stack and remove its volumes
airgap-down:
    docker compose -f deploy/scenarios/airgap/docker-compose.airgap.yml down -v

# ONE-COMMAND air-gap proof. Ensures the sealed stack is up (builds the image
# only if it is missing), warm-builds the loadtest crate so the run emits no
# compiler spew, runs all three assertion scripts (01 egress, 02 load,
# 03 governance) WITHOUT dying on the first failure, then prints a single
# PASS/FAIL summary. Leaves the stack up for inspection by default — pass
# TEARDOWN=true to remove it (and its volumes) at the end.
#
#   just airgap                # run, leave stack up
#   just airgap TEARDOWN=true  # run, then tear down
airgap TEARDOWN="false":
    #!/usr/bin/env bash
    set -uo pipefail
    COMPOSE_FILE="deploy/scenarios/airgap/docker-compose.airgap.yml"
    PORT="${AIRGAP_HTTP_PORT:-8090}"
    LOADTEST_MANIFEST="../systemprompt-core/crates/tests/loadtest/Cargo.toml"

    # 1. Ensure the stack is up. Build the image only if it is not present yet
    #    (a first-time build pulls in ../systemprompt-core and takes ~10 min).
    if curl -fsS -o /dev/null --max-time 3 "http://localhost:${PORT}/api/v1/health" 2>/dev/null; then
      echo "  air-gap stack already healthy on :${PORT}"
    else
      if docker compose -f "$COMPOSE_FILE" config --images 2>/dev/null \
         | xargs -r -I{} docker image inspect {} >/dev/null 2>&1; then
        echo "  air-gap image present — starting stack (no rebuild)"
        docker compose -f "$COMPOSE_FILE" up -d
      else
        echo "  air-gap image missing — building stack (first run, ~10 min)"
        docker compose -f "$COMPOSE_FILE" up -d --build
      fi
      echo "  waiting for app healthcheck on :${PORT} ..."
      for i in $(seq 1 120); do
        if curl -fsS -o /dev/null "http://localhost:${PORT}/api/v1/health" 2>/dev/null; then
          echo "  app healthy after ${i}s"
          break
        fi
        sleep 1
      done
    fi

    # 2. Warm-build the loadtest crate quietly so STEP 02's `cargo run` emits no
    #    build output mid-demo. Non-fatal: 02-load.sh re-checks the manifest.
    if [[ -f "$LOADTEST_MANIFEST" ]]; then
      echo "  warm-building the loadtest crate ..."
      cargo build --quiet --manifest-path "$LOADTEST_MANIFEST" 2>/dev/null || true
    else
      echo "  loadtest crate not found at ${LOADTEST_MANIFEST} — skipping warm-build" >&2
      echo "  (it is unpublished systemprompt-core dev tooling; 02-load.sh will build it on demand if present)" >&2
    fi

    # 3. Run all three scripts, capturing each exit code (do NOT stop on first
    #    failure — the operator must see the full picture).
    declare -A RESULT
    for s in 01-egress-assert 02-load 03-governance; do
      echo ""
      if "./demo/scenarios/airgap/${s}.sh"; then
        RESULT[$s]="PASS"
      else
        RESULT[$s]="FAIL"
      fi
    done

    # 4. Single PASS/FAIL summary.
    echo ""
    echo "══════════════════════════════════════════════════════════"
    echo "  AIR-GAP PROOF SUMMARY"
    echo "══════════════════════════════════════════════════════════"
    OVERALL=0
    for s in 01-egress-assert 02-load 03-governance; do
      printf "  %-22s %s\n" "$s" "${RESULT[$s]}"
      [[ "${RESULT[$s]}" == "PASS" ]] || OVERALL=1
    done
    echo "══════════════════════════════════════════════════════════"
    [[ "$OVERALL" -eq 0 ]] && echo "  RESULT: PASS" || echo "  RESULT: FAIL"

    # 5. Optional teardown.
    if [[ "{{TEARDOWN}}" == "true" ]]; then
      echo ""
      echo "  TEARDOWN=true — removing the air-gap stack and volumes"
      just airgap-down
    fi

    exit "$OVERALL"

# Run the air-gap demo scripts in sequence, stopping on first failure.
# Policies (quotas/safety) ship as services/gateway/policies.yaml and are
# ingested by the publish_pipeline job at server boot. Model exposure lives
# in the profile provider registry (profile.providers in
# .systemprompt/profiles/airgap/profile.yaml).
airgap-test:
    #!/usr/bin/env bash
    set -euo pipefail
    ./demo/scenarios/airgap/01-egress-assert.sh
    ./demo/scenarios/airgap/02-load.sh
    ./demo/scenarios/airgap/03-governance.sh

# Reproducibility proof: tear down (incl. volumes), bring back up reusing the
# already-built image, run the full assertion suite from zero state. Prints
# wall-clock time. Use this in front of a reviewer who wants to see the demo
# work from a clean container + clean database, without a 10-minute image
# rebuild. Image-level reproducibility is a separate concern — see
# demo/scenarios/airgap/architecture.md §9 (the [patch.crates-io] block
# requires systemprompt-core >= 0.10.4 to be published before the image can
# be rebuilt from this repo in isolation).
airgap-fresh-test:
    #!/usr/bin/env bash
    set -euo pipefail
    COMPOSE_FILE="deploy/scenarios/airgap/docker-compose.airgap.yml"
    # Refuse to run if the image isn't already built — the rebuild path needs
    # the sibling systemprompt-core repo and a 10-minute window, and silently
    # falling into that on a demo machine is a bad surprise.
    if ! docker image inspect airgap-app >/dev/null 2>&1 \
       && ! docker compose -f "$COMPOSE_FILE" config --images 2>/dev/null | head -1 | xargs -I{} docker image inspect {} >/dev/null 2>&1; then
      echo "ERROR: app image not present. First-time build needed:" >&2
      echo "  just airgap-up   # builds the image (~10 min, needs ../systemprompt-core)" >&2
      exit 1
    fi
    START=$(date +%s)
    just airgap-down
    # No --build: reuse the existing image. This is the from-zero DATA reset,
    # not the from-zero BUILD reset.
    docker compose -f "$COMPOSE_FILE" up -d
    echo "Waiting for app healthcheck..."
    for i in $(seq 1 120); do
      if curl -fsS -o /dev/null "http://localhost:${AIRGAP_HTTP_PORT:-8090}/api/v1/health" 2>/dev/null; then
        echo "App healthy after ${i}s"
        break
      fi
      sleep 1
    done
    just airgap-test
    END=$(date +%s)
    echo ""
    echo "═══════════════════════════════════════════════════════"
    echo "  FRESH AIR-GAP RUN COMPLETE in $((END - START))s"
    echo "═══════════════════════════════════════════════════════"

# ══════════════════════════════════════════════════════════════════════════════
# SCALED / DISTRIBUTED SCENARIO
# ══════════════════════════════════════════════════════════════════════════════

# Bring up the multi-replica scaled stack (postgres primary/replica + N app replicas + 1 scheduler + nginx LB)
scaled-up REPLICAS="3":
    #!/usr/bin/env bash
    set -euo pipefail
    # Stage the host-built binaries into a real dir inside the build context —
    # `target` is a symlink to a shared cargo cache that buildkit can't follow.
    if [[ ! -x target/release/systemprompt || ! -x target/release/systemprompt-mcp-agent ]]; then
        echo "ERROR: release binaries missing. Run: just build --release" >&2
        exit 1
    fi
    mkdir -p deploy/scenarios/scaled/.bin
    cp -L target/release/systemprompt           deploy/scenarios/scaled/.bin/systemprompt
    cp -L target/release/systemprompt-mcp-agent deploy/scenarios/scaled/.bin/systemprompt-mcp-agent
    docker compose -f deploy/scenarios/scaled/docker-compose.scaled.yml up -d --build --scale app={{REPLICAS}}

# Tear down the scaled stack and remove its volumes
scaled-down:
    docker compose -f deploy/scenarios/scaled/docker-compose.scaled.yml down -v

# ONE COMMAND: reset → build → up → wait-for-health → mint token → run all fast
# proofs → capture logs → single verdict. Leaves the stack up by default.
#   just scaled-demo                # 3 replicas, stack left up
#   REPLICAS=5 just scaled-demo     # scale wider
#   KEEP=0 just scaled-demo         # tear down at the end
#   SOAK=1 just scaled-demo         # also run the ~1h soak (long!)
scaled-demo:
    #!/usr/bin/env bash
    set -uo pipefail
    chmod +x demo/scenarios/scaled/run.sh
    ./demo/scenarios/scaled/run.sh

# Run the scaled demo scripts in sequence against an ALREADY-RUNNING stack.
# Prefer `just scaled-demo` (full lifecycle). Use this only when the stack is
# already up and healthy. Skips 02-soak.sh — the long (~1h) sustained soak; run
# it on its own when needed: ./demo/scenarios/scaled/02-soak.sh
scaled-test:
    #!/usr/bin/env bash
    set -euo pipefail
    chmod +x demo/scenarios/scaled/01-load.sh \
             demo/scenarios/scaled/03-replica-distribution.sh \
             demo/scenarios/scaled/04-scheduler-isolation.sh
    ./demo/scenarios/scaled/01-load.sh
    ./demo/scenarios/scaled/03-replica-distribution.sh
    ./demo/scenarios/scaled/04-scheduler-isolation.sh

# ══════════════════════════════════════════════════════════════════════════════
# ADMIN & PLUGINS
# ══════════════════════════════════════════════════════════════════════════════

# Generate WebAuthn setup token for admin user
webauthn-admin EMAIL:
    {{CLI}} admin users webauthn generate-setup-token --email "{{EMAIL}}"

# Generate plugin output
marketplace:
    {{CLI}} core plugins generate

# Update Anthropic official plugins from vendor submodule and reimport
update-anthropic-plugins:
    git submodule update --remote vendor/knowledge-work-plugins
    {{CLI}} infra jobs run import_anthropic_plugins

# ══════════════════════════════════════════════════════════════════════════════
# TERMINAL RECORDINGS (README SVGs)
# ══════════════════════════════════════════════════════════════════════════════

# Regenerate terminal SVG recordings. Pass numbers to limit scope, e.g. `just record-svgs 3 7`.
record-svgs *NUMBERS:
    ./demo/recording/svg/record.sh {{NUMBERS}}

# ══════════════════════════════════════════════════════════════════════════════
# BENCHMARKS
# ══════════════════════════════════════════════════════════════════════════════

# Benchmark governance endpoint. Downloads `hey` for the host OS/arch on first run.
benchmark REQUESTS="200" CONCURRENCY="100":
    #!/usr/bin/env bash
    set -e
    # Use system hey if available, else /tmp/hey
    if command -v hey >/dev/null 2>&1; then
        HEY="$(command -v hey)"
    else
        HEY="/tmp/hey"
    fi
    # Re-download if the cached binary can't execute here (e.g. Linux hey on a Mac).
    if ! { [[ -x "$HEY" ]] && "$HEY" --help >/dev/null 2>&1; }; then
        rm -f "$HEY"
        HEY="/tmp/hey"
        OS_ARCH="$(uname -s)/$(uname -m)"
        case "$OS_ARCH" in
            Darwin/*)
                HEY_URL="https://hey-release.s3.us-east-2.amazonaws.com/hey_darwin_amd64"
                echo "Installing hey from $HEY_URL..."
                curl -fsSL "$HEY_URL" -o "$HEY" && chmod +x "$HEY"
                ;;
            Linux/x86_64|Linux/amd64)
                HEY_URL="https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64"
                echo "Installing hey from $HEY_URL..."
                if ! curl -fsSL "$HEY_URL" -o "$HEY"; then
                    echo "ERROR: failed to download hey. Run: sudo apt-get install hey" >&2; exit 1
                fi
                chmod +x "$HEY"
                ;;
            *) echo "ERROR: no prebuilt hey for $OS_ARCH. Install with 'brew install hey' or 'go install github.com/rakyll/hey@latest'." >&2; exit 1 ;;
        esac
        if ! "$HEY" --help >/dev/null 2>&1; then
            echo "ERROR: hey won't run on $OS_ARCH." >&2
            if [[ "$OS_ARCH" == "Darwin/arm64" ]]; then
                echo "       Apple Silicon: 'softwareupdate --install-rosetta' or 'brew install hey'." >&2
            else
                echo "       Install manually: 'sudo apt-get install hey' or 'go install github.com/rakyll/hey@latest'." >&2
            fi
            rm -f "$HEY"; exit 1
        fi
    fi
    TOKEN_FILE="demo/.token"
    if [[ ! -f "$TOKEN_FILE" ]]; then
        echo "ERROR: No token. Run: ./demo/00-preflight.sh" >&2
        exit 1
    fi
    TOKEN=$(cat "$TOKEN_FILE")
    echo "Governance endpoint: {{REQUESTS}} requests, {{CONCURRENCY}} concurrent"
    echo ""
    "$HEY" -n {{REQUESTS}} -c {{CONCURRENCY}} -m POST \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"bench","tool_input":{"file_path":"/src/main.rs"}}' \
        "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo"

# Syntax-check install.sh (install.sh is the user-facing installer)
install-sh-test:
    bash -n scripts/install.sh
    shellcheck scripts/install.sh 2>/dev/null || echo "(install shellcheck to lint: apt install shellcheck)"

# Check the Nix flake builds + runs
flake-check:
    nix flake check
    nix run .# -- --version
