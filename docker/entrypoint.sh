#!/bin/sh
# Container entrypoint for systemprompt-template.
# Generates profile + secrets from env vars on first boot,
# waits for Postgres, runs migrations, starts the server.
set -eu

PROFILE_DIR="/app/services/profiles/docker"
PROFILE_FILE="$PROFILE_DIR/profile.yaml"
SECRETS_FILE="$PROFILE_DIR/secrets.json"

if [ -z "${ANTHROPIC_API_KEY:-}" ] && [ -z "${OPENAI_API_KEY:-}" ] && [ -z "${GEMINI_API_KEY:-}" ]; then
    echo "ERROR: set at least one of ANTHROPIC_API_KEY, OPENAI_API_KEY, GEMINI_API_KEY in .env" >&2
    exit 1
fi

mkdir -p "$PROFILE_DIR"

if [ ! -f "$PROFILE_FILE" ]; then
    echo "Writing profile.yaml..."
    cat > "$PROFILE_FILE" <<YAML
name: docker
display_name: Docker
target: local
site:
  name: systemprompt.io
  github_link: null
database:
  type: postgres
  external_db_access: false
server:
  host: 0.0.0.0
  port: 8080
  api_server_url: http://localhost:${HTTP_PORT:-8080}
  api_internal_url: http://localhost:8080
  api_external_url: http://localhost:${HTTP_PORT:-8080}
  use_https: false
  cors_allowed_origins:
  - http://localhost:${HTTP_PORT:-8080}
paths:
  system: /app
  services: /app/services
  bin: /app/bin
  web_path: /app/web
  storage: /app/storage
  geoip_database: null
security:
  jwt_issuer: systemprompt-docker
  jwt_access_token_expiration: 2592000
  jwt_refresh_token_expiration: 15552000
  jwt_audiences: [web, api, a2a, mcp]
rate_limits:
  disabled: true
  oauth_public_per_second: 10
  oauth_auth_per_second: 10
  contexts_per_second: 100
  tasks_per_second: 50
  artifacts_per_second: 50
  agent_registry_per_second: 50
  agents_per_second: 20
  mcp_registry_per_second: 50
  mcp_per_second: 200
  stream_per_second: 100
  content_per_second: 50
  burst_multiplier: 3
  tier_multipliers:
    admin: 10.0
    user: 1.0
    a2a: 5.0
    mcp: 5.0
    service: 5.0
    anon: 0.5
runtime:
  environment: development
  log_level: verbose
  output_format: text
  no_color: false
  non_interactive: true
cloud:
  tenant_id: null
  validation: warn
secrets:
  secrets_path: ./secrets.json
  validation: warn
  source: file
extensions:
  disabled: []
gateway:
  enabled: true
  routes:
    - model_pattern: "claude-*"
      provider: anthropic
      endpoint: https://api.anthropic.com/v1
      api_key_secret: anthropic
    - model_pattern: "gpt-*"
      provider: openai
      endpoint: https://api.openai.com/v1
      api_key_secret: openai
    - model_pattern: "gemini-*"
      provider: gemini
      endpoint: https://generativelanguage.googleapis.com/v1beta
      api_key_secret: gemini
YAML
fi

if [ ! -f "$SECRETS_FILE" ]; then
    echo "Writing secrets.json..."
    JWT_SECRET="${JWT_SECRET:-$(head -c 48 /dev/urandom | base64 | tr -d '+/=' | head -c 64)}"
    jq -n \
        --arg jwt "$JWT_SECRET" \
        --arg db "${DATABASE_URL}" \
        --arg anthropic "${ANTHROPIC_API_KEY:-}" \
        --arg openai "${OPENAI_API_KEY:-}" \
        --arg gemini "${GEMINI_API_KEY:-}" \
        '{
            jwt_secret: $jwt,
            database_url: $db,
            anthropic: (if $anthropic == "" then null else $anthropic end),
            openai:    (if $openai    == "" then null else $openai    end),
            gemini:    (if $gemini    == "" then null else $gemini    end)
        }' > "$SECRETS_FILE"
    chmod 600 "$SECRETS_FILE"
fi

export SYSTEMPROMPT_PROFILE="$PROFILE_FILE"

echo "Waiting for Postgres..."
i=0
until pg_isready -h postgres -U systemprompt -d systemprompt >/dev/null 2>&1; do
    i=$((i + 1))
    if [ "$i" -ge 60 ]; then
        echo "ERROR: Postgres did not become ready within 60s." >&2
        exit 1
    fi
    sleep 1
done
echo "Postgres is ready."

echo "Running database migrations..."
/app/bin/systemprompt infra db migrate

echo "Starting services..."
exec /app/bin/systemprompt infra services start --foreground
