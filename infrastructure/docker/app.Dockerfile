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
COPY core/crates/modules/log/schema /app/core/crates/modules/log/schema
COPY core/crates/modules/mcp/schema /app/core/crates/modules/mcp/schema
COPY core/crates/modules/oauth/schema /app/core/crates/modules/oauth/schema
COPY core/crates/modules/scheduler/schema /app/core/crates/modules/scheduler/schema
COPY core/crates/modules/users/schema /app/core/crates/modules/users/schema

# Copy module queries (seed data, SQL files)
COPY core/crates/modules/agent/src/queries /app/core/crates/modules/agent/src/queries
COPY core/crates/modules/ai/src/queries /app/core/crates/modules/ai/src/queries
COPY core/crates/modules/blog/src/queries /app/core/crates/modules/blog/src/queries
COPY core/crates/modules/oauth/src/queries /app/core/crates/modules/oauth/src/queries

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

# Copy templates (both oauth and web prerendering)
COPY core/templates /app/core/templates
COPY core/web/templates /app/core/web/templates

# Copy service configuration
COPY config /app/config
COPY crates/services/config /app/crates/services/config
COPY crates/services/ai /app/crates/services/ai
COPY crates/services/agents /app/crates/services/agents
COPY crates/services/mcp /app/crates/services/mcp
COPY crates/services/content /app/crates/services/content
COPY crates/services/web /app/crates/services/web
COPY crates/services/skills /app/crates/services/skills

# Copy web dist
COPY core/web/dist /app/core/web/dist

# Copy GeoIP database for IP geolocation
# Note: Production uses volume mount (auto-downloaded by startup.sh)
# For local builds, ensure data/GeoLite2-City.mmdb exists or download from:
#   https://download.db-ip.com/free/dbip-city-lite-YYYY-MM.mmdb.gz
COPY data/GeoLite2-City.mmdb /app/data/GeoLite2-City.mmdb

# Make binaries executable and fix permissions
RUN chmod +x /app/target/release/* && \
    mkdir -p /app/core/web/dist/sitemaps && \
    chown -R app:app /app && \
    chmod 755 /app/logs /app/storage /app/storage/generated_images && \
    chmod -R a+rX /app/crates/services/content

USER app

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

# Environment variables (can be overridden by docker-compose)
ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    PATH="/app/target/release:$PATH" \
    CARGO_TARGET_DIR=/app/target \
    SYSTEM_PATH=/app \
    WEB_DIR=/app/core/web/dist \
    SYSTEMPROMPT_CONFIG_PATH=/app/crates/services/config/config.yml \
    AI_CONFIG_PATH=/app/config/ai.yaml

# Startup script with fail-fast validation
RUN echo '#!/bin/bash\n\
set -e\n\
\n\
echo "🚀 Starting SystemPrompt..."\n\
\n\
# Step 1: Database migrations (MUST succeed)\n\
echo "📊 Running database migrations..."\n\
if ! /app/target/release/systemprompt db migrate; then\n\
    echo "❌ FATAL: Database migrations failed"\n\
    exit 1\n\
fi\n\
echo "✅ Database migrations completed"\n\
\n\
# Step 2: Cleanup stale services\n\
echo "🧹 Cleaning up stale service entries..."\n\
if ! /app/target/release/systemprompt cleanup-services; then\n\
    echo "⚠️  Warning: Service cleanup had issues (non-fatal)"\n\
fi\n\
\n\
# Step 3: Start API server (scheduler will run jobs automatically)\n\
echo "🌐 Starting API server..."\n\
exec /app/target/release/systemprompt serve api --foreground\n\
' > /app/start.sh && \
    chmod +x /app/start.sh

CMD ["/app/start.sh"]
