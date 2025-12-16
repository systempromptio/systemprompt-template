FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libpq5 \
    libssl3 \
    lsof \
    procps \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 app
WORKDIR /app

# Create directory structure
RUN mkdir -p /app/target/release /app/core/crates/modules /app/data /app/logs /app/storage/generated_images

# Copy Cargo workspace files (for module discovery)
COPY Cargo.lock Cargo.toml /app/
COPY core/Cargo.lock /app/core/

# Copy pre-built binaries from staging directory (all binaries dynamically built)
COPY infrastructure/build-context/release/* /app/target/release/

# Copy module schemas (only those that exist)
COPY core/crates/modules/agent/schema /app/core/crates/modules/agent/schema
COPY core/crates/modules/ai/schema /app/core/crates/modules/ai/schema
COPY core/crates/modules/blog/schema /app/core/crates/modules/blog/schema
COPY core/crates/modules/config/schema /app/core/crates/modules/config/schema
COPY core/crates/modules/core/schema /app/core/crates/modules/core/schema
COPY core/crates/modules/database/schema /app/core/crates/modules/database/schema
COPY core/crates/modules/log/schema /app/core/crates/modules/log/schema
COPY core/crates/modules/mcp/schema /app/core/crates/modules/mcp/schema
COPY core/crates/modules/oauth/schema /app/core/crates/modules/oauth/schema
COPY core/crates/modules/scheduler/schema /app/core/crates/modules/scheduler/schema
COPY core/crates/modules/users/schema /app/core/crates/modules/users/schema

# Copy module queries (seed data, SQL files)
COPY core/crates/modules/ai/src/queries /app/core/crates/modules/ai/src/queries
COPY core/crates/modules/blog/src/queries /app/core/crates/modules/blog/src/queries
COPY core/crates/modules/oauth/src/queries /app/core/crates/modules/oauth/src/queries
COPY core/crates/modules/core/src/queries /app/core/crates/modules/core/src/queries
COPY core/crates/modules/users/src/queries /app/core/crates/modules/users/src/queries

# Copy module.yml files
COPY core/crates/modules/agent/module.yml /app/core/crates/modules/agent/
COPY core/crates/modules/ai/module.yml /app/core/crates/modules/ai/
COPY core/crates/modules/api/module.yml /app/core/crates/modules/api/
COPY core/crates/modules/blog/module.yml /app/core/crates/modules/blog/
COPY core/crates/modules/config/module.yml /app/core/crates/modules/config/
COPY core/crates/modules/core/module.yml /app/core/crates/modules/core/
COPY core/crates/modules/database/module.yml /app/core/crates/modules/database/
COPY core/crates/modules/log/module.yml /app/core/crates/modules/log/
COPY core/crates/modules/mcp/module.yml /app/core/crates/modules/mcp/
COPY core/crates/modules/oauth/module.yml /app/core/crates/modules/oauth/
COPY core/crates/modules/scheduler/module.yml /app/core/crates/modules/scheduler/
COPY core/crates/modules/users/module.yml /app/core/crates/modules/users/

# Copy templates (oauth)
COPY core/templates /app/core/templates

# Copy service configuration
COPY config /app/config
COPY services/config /app/services/config
COPY services/ai /app/services/ai
COPY services/agents /app/services/agents
COPY services/mcp /app/services/mcp
COPY services/content /app/services/content
COPY services/web /app/services/web
COPY services/skills /app/services/skills
COPY services/scheduler /app/services/scheduler
COPY services/profiles /app/services/profiles

# Copy web dist
COPY core/web/dist /app/core/web/dist

# Copy GeoIP database for IP geolocation (optional)
# Note: Production uses volume mount (auto-downloaded by startup.sh)
# For local builds, ensure data/GeoLite2-City.mmdb exists or download from:
#   https://download.db-ip.com/free/dbip-city-lite-YYYY-MM.mmdb.gz
# COPY data/GeoLite2-City.mmdb /app/data/GeoLite2-City.mmdb

# Make binaries executable and fix permissions
RUN chmod +x /app/target/release/* && \
    mkdir -p /app/core/web/dist/sitemaps && \
    chown -R app:app /app && \
    chmod 755 /app/logs /app/storage /app/storage/generated_images && \
    chmod -R a+rX /app/services/content

USER app

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

# Infrastructure env vars (fixed for this container)
ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    PATH="/app/target/release:$PATH" \
    CARGO_TARGET_DIR=/app/target \
    SYSTEM_PATH=/app \
    SYSTEMPROMPT_SERVICES_PATH=/app/services \
    SYSTEMPROMPT_PROFILE=prod

# Startup script with fail-fast validation
RUN echo '#!/bin/bash\n\
set -e\n\
\n\
echo "🚀 Starting SystemPrompt..."\n\
\n\
# Step 0: Validate template is complete\n\
echo "🔍 Validating template..."\n\
PROFILE="${SYSTEMPROMPT_PROFILE:-prod}"\n\
PROFILE_FILE="/app/services/profiles/${PROFILE}.profile.yml"\n\
\n\
if [ ! -f "$PROFILE_FILE" ]; then\n\
    echo "❌ FATAL: Profile not found: $PROFILE_FILE"\n\
    echo "   The template must include a valid profile at services/profiles/${PROFILE}.profile.yml"\n\
    exit 1\n\
fi\n\
\n\
# Validate required config files exist\n\
REQUIRED_FILES=(\n\
    "/app/services/config/config.yml"\n\
    "/app/services/content/config.yml"\n\
    "/app/services/web/config.yml"\n\
    "/app/services/ai/config.yml"\n\
)\n\
for f in "${REQUIRED_FILES[@]}"; do\n\
    if [ ! -f "$f" ]; then\n\
        echo "❌ FATAL: Required config missing: $f"\n\
        exit 1\n\
    fi\n\
done\n\
\n\
# Validate required secrets are set\n\
REQUIRED_SECRETS=("DATABASE_URL" "JWT_SECRET" "ADMIN_PASSWORD")\n\
for secret in "${REQUIRED_SECRETS[@]}"; do\n\
    if [ -z "${!secret}" ]; then\n\
        echo "❌ FATAL: Required secret not set: $secret"\n\
        exit 1\n\
    fi\n\
done\n\
echo "✅ Template validation passed"\n\
\n\
# Step 1: Export env vars from profile\n\
echo "📋 Loading profile: $PROFILE"\n\
export SYSTEMPROMPT_CONFIG_PATH=/app/services/config/config.yml\n\
export CONTENT_CONFIG_PATH=/app/services/content/config.yml\n\
export SYSTEMPROMPT_WEB_CONFIG_PATH=/app/services/web/config.yml\n\
export SYSTEMPROMPT_WEB_METADATA_PATH=/app/services/web/metadata.yml\n\
export AI_CONFIG_PATH=/app/config/ai.yaml\n\
export SYSTEMPROMPT_SKILLS_PATH=/app/services/skills\n\
export STORAGE_PATH="${STORAGE_PATH:-/services}"\n\
export WEB_DIR=/app/core/web/dist\n\
export SITENAME="${SITENAME:-SystemPrompt}"\n\
export GITHUB_LINK="${GITHUB_LINK:-}"\n\
export API_EXTERNAL_URL="${API_EXTERNAL_URL:-http://localhost:8080}"\n\
export CORS_ALLOWED_ORIGINS="${CORS_ALLOWED_ORIGINS:-$API_EXTERNAL_URL}"\n\
\n\
# Step 2: Database migrations (MUST succeed)\n\
echo "📊 Running database migrations..."\n\
if ! /app/target/release/systemprompt db migrate; then\n\
    echo "❌ FATAL: Database migrations failed"\n\
    exit 1\n\
fi\n\
echo "✅ Database migrations completed"\n\
\n\
# Step 3: Cleanup stale services\n\
echo "🧹 Cleaning up stale service entries..."\n\
if ! /app/target/release/systemprompt cleanup-services; then\n\
    echo "⚠️  Warning: Service cleanup had issues (non-fatal)"\n\
fi\n\
\n\
# Step 4: Start API server (scheduler will run jobs automatically)\n\
echo "🌐 Starting API server..."\n\
exec /app/target/release/systemprompt serve api --foreground\n\
' > /app/start.sh && \
    chmod +x /app/start.sh

CMD ["/app/start.sh"]
