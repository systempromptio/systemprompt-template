#!/usr/bin/env bash
# Local install of one flavour: Docker Postgres, the flavour's build, a profile
# at .systemprompt/profiles/<flavour>/, its staged services tree, migrations,
# the bootstrap admin, then the publish pipeline (a flavour with a site) or the
# console asset jobs (every other flavour).
#   scripts/flavour-setup.sh FLAVOUR [--anthropic-key K] [--openai-key K]
#       [--gemini-key K] [--http-port N] [--pg-port N] [--admin-email E]
#       [--no-build] [--dry-run]
# A bare re-run keeps the ports chosen at first setup. ADMIN_EMAIL defaults to
# `git config user.email`. --no-build uses binaries already in
# target/flavour-<flavour>/ (release, else debug). --dry-run prints each step.
# A desktop flavour (artefact: desktop) installs no server: its build arrives
# in stage S1 of the harness programme, and until then this says so and exits 0.
set -euo pipefail
source "$(dirname "$0")/lib/flavour.sh"
usage() { sed -n '2,11p' "$0" | sed 's/^# \{0,1\}//' >&2; exit 2; }
flavour="${1:-}"
[ -n "$flavour" ] && [ "${flavour#-}" = "$flavour" ] || usage
shift
flavour_require "$flavour"
if [ "$(flavour_get "$flavour" artefact)" = desktop ]; then
    echo "flavour $flavour is a desktop artefact, not an image: the desktop build arrives in stage S1 of the harness programme, so there is nothing to set up yet"
    exit 0
fi
anthropic_key="" openai_key="" gemini_key="" http_port="" pg_port="" admin_email=""
build=true dry_run=false
while [ $# -gt 0 ]; do
    case "$1" in
        --anthropic-key) anthropic_key="${2:?}"; shift 2 ;;
        --openai-key) openai_key="${2:?}"; shift 2 ;;
        --gemini-key) gemini_key="${2:?}"; shift 2 ;;
        --http-port) http_port="${2:?}"; shift 2 ;;
        --pg-port) pg_port="${2:?}"; shift 2 ;;
        --admin-email) admin_email="${2:?}"; shift 2 ;;
        --no-build) build=false; shift ;;
        --dry-run) dry_run=true; shift ;;
        *) echo "unknown argument: $1" >&2; usage ;;
    esac
done

root="$FLAVOUR_ROOT"
cd "$root"
profile_dir="$root/.systemprompt/profiles/$flavour"
profile_file="$profile_dir/profile.yaml"
compose_file="$root/.systemprompt/docker/$flavour.yaml"
target_dir="$root/target/flavour-$flavour"
services_dir="$root/.systemprompt/flavours/$flavour/services"
requires_ai_key="$(flavour_get "$flavour" requires_ai_key)"
site="$(flavour_get "$flavour" site)"
project="$(just --justfile "$root/justfile" _project_name "$flavour")"
export SYSTEMPROMPT_PROFILE="$profile_file"

run() {
    if [ "$dry_run" = true ]; then
        printf '[dry-run]'; printf ' %q' "$@"; printf '\n'
    else
        "$@"
    fi
}
step() { echo "==> $*"; }

admin_email="${admin_email:-$(git config user.email 2>/dev/null || true)}"
admin_email="${admin_email:-admin@localhost.localdomain}"
if [ -z "$http_port" ]; then
    http_port="$( [ -f "$profile_file" ] && sed -n 's/^ *port: //p' "$profile_file" | head -1 || true)"
    http_port="${http_port:-8080}"
fi
if [ -z "$pg_port" ]; then
    pg_port="$( [ -f "$compose_file" ] && sed -n 's/.*"\([0-9][0-9]*\):5432".*/\1/p' "$compose_file" | head -1 || true)"
    pg_port="${pg_port:-5432}"
fi

has_key=false
if [ -n "$anthropic_key$openai_key$gemini_key" ]; then has_key=true; fi
if [ "$requires_ai_key" = true ] && [ "$has_key" = false ] && [ ! -f "$profile_dir/secrets.json" ] && [ ! -t 0 ]; then
    echo "setup $flavour needs an AI provider key: not on a TTY, so the provider menu cannot be shown." >&2
    echo "  just setup $flavour --anthropic-key <key>   (or --openai-key / --gemini-key)" >&2
    exit 1
fi
if [ "$requires_ai_key" = false ] && [ "$has_key" = true ]; then
    echo "setup $flavour: this flavour links no AI provider; the supplied keys are ignored." >&2
    has_key=false
fi

if [ "$build" = true ]; then
    step "Building $flavour (features $(flavour_py features "$flavour")) into $target_dir"
    run env FLAVOUR="$flavour" just --justfile "$root/justfile" build --release
fi
bin=""
for candidate in "$target_dir/release/systemprompt" "$target_dir/debug/systemprompt"; do
    if [ -x "$candidate" ]; then bin="$candidate"; break; fi
done
if [ -z "$bin" ]; then
    [ "$dry_run" = true ] || { echo "no systemprompt binary in $target_dir; run without --no-build" >&2; exit 1; }
    bin="$target_dir/release/systemprompt"
fi

step "Staging the $flavour services tree at $services_dir"
run bash "$root/scripts/flavour-stage-services.sh" "$flavour" "$services_dir" --link

mkdir -p "$(dirname "$compose_file")"
if [ -f "$compose_file" ] && ! grep -q "\"${pg_port}:5432\"" "$compose_file"; then
    step "Postgres compose pins another host port; recreating it on $pg_port"
    run docker compose -p "$project" -f "$compose_file" down
    run rm -f "$compose_file"
fi
if [ ! -f "$compose_file" ] && [ "$dry_run" = false ]; then
    cat > "$compose_file" <<YAML
services:
  postgres:
    image: postgres:18-alpine
    restart: unless-stopped
    environment:
      POSTGRES_USER: systemprompt
      POSTGRES_PASSWORD: 123
      POSTGRES_DB: systemprompt
    ports:
      - "${pg_port}:5432"
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
step "Starting Postgres ($project, host port $pg_port)"
run docker compose -p "$project" -f "$compose_file" up -d --wait

if [ ! -f "$profile_file" ]; then
    step "Generating the $flavour profile with admin setup"
    setup_args=(admin setup --no-migrate --environment "$flavour"
        --db-host localhost --db-port "$pg_port"
        --db-user systemprompt --db-password 123 --db-name systemprompt
        --admin-email "$admin_email")
    if [ "$has_key" = true ]; then
        default_provider=""
        if [ -n "$anthropic_key" ]; then setup_args+=(--anthropic-key "$anthropic_key"); default_provider="${default_provider:-anthropic}"; fi
        if [ -n "$openai_key" ]; then setup_args+=(--openai-key "$openai_key"); default_provider="${default_provider:-openai}"; fi
        if [ -n "$gemini_key" ]; then setup_args+=(--gemini-key "$gemini_key"); default_provider="${default_provider:-gemini}"; fi
        run "$bin" "${setup_args[@]}" --yes --default-provider "$default_provider"
    elif [ "$requires_ai_key" = true ]; then
        run env SYSTEMPROMPT_NON_INTERACTIVE=1 "$bin" "${setup_args[@]}"
    else
        run "$bin" "${setup_args[@]}" --yes
    fi
    if [ "$http_port" != 8080 ]; then
        step "Re-pointing server, governance hook, JWT issuer and CORS at port $http_port"
        url="http://localhost:$http_port"
        run "$bin" admin config server set --port "$http_port" \
            --api-server-url "$url" --api-internal-url "$url" --api-external-url "$url"
        run "$bin" admin config governance set --mode webhook --url "$url/api/public/govern/authz"
        run "$bin" admin config security set --jwt-issuer "$url"
        run "$bin" admin config server cors add "$url"
        run "$bin" admin config server cors add "http://127.0.0.1:$http_port"
        run "$bin" admin config server cors remove "http://localhost:8080"
        run "$bin" admin config server cors remove "http://127.0.0.1:8080"
    fi
elif [ "$has_key" = true ]; then
    echo "setup $flavour: $profile_file exists, so admin setup was skipped and the keys were NOT applied." >&2
    echo "  To re-provision: rm -rf \"$profile_dir\" && just setup $flavour <keys...>" >&2
fi

step "Applying flavours/$flavour/profile.overlay.yaml"
run bash "$root/scripts/flavour-overlay.sh" "$flavour" "$profile_file" "$bin" \
    "paths.services=$services_dir" "paths.bin=$(dirname "$bin")"

mkdir -p "$root/web/dist"
step "Running database migrations"
run "$bin" infra db migrate
step "Ensuring the bootstrap admin user ($admin_email)"
run "$bin" admin bootstrap --email "$admin_email"
if [ ! -f "$root/signing_key.pem" ]; then
    step "Generating the JWT signing key"
    run "$bin" admin keys generate --output "$root/signing_key.pem"
fi
if [ "$site" = true ]; then
    step "Publishing the site"
    run "$bin" infra jobs run publish_pipeline
else
    step "Publishing the console assets"
    run "$bin" infra jobs run bundle_admin_css
    run "$bin" infra jobs run copy_extension_assets
fi
echo ""
echo "Setup of $flavour complete. Run: FLAVOUR=$flavour just start"
