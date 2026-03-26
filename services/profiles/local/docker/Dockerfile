# foodles.com Application Dockerfile
# Built by: systemprompt cloud profile create
# Used by: systemprompt cloud deploy

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    libpq5 \
    lsof \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 app
WORKDIR /app

RUN mkdir -p /app/bin /app/logs /app/storage/files/images /app/storage/files/images/generated /app/storage/files/images/logos /app/storage/files/audio /app/storage/files/video /app/storage/files/documents /app/storage/files/uploads /app/web

# Copy pre-built binaries
COPY target/release/systemprompt /app/bin/

# Copy MCP server binaries
COPY target/release/systemprompt-mcp-agent /app/bin/

# Copy storage assets (images, etc.)
COPY storage /app/storage

# Copy web dist (generated HTML, CSS, JS)
COPY web/dist /app/web/dist

# Copy extension assets
COPY extensions/mcp/systemprompt /app/extensions/mcp/systemprompt

# Copy services configuration
COPY services /app/services

# Copy profiles
COPY .systemprompt/profiles /app/services/profiles

RUN chmod +x /app/bin/* && chown -R app:app /app

USER app
EXPOSE 8080

# Environment configuration
ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    PATH="/app/bin:$PATH" \
    SYSTEMPROMPT_PROFILE=/app/services/profiles/local/profile.yaml \
    SYSTEMPROMPT_SERVICES_PATH=/app/services \
    SYSTEMPROMPT_TEMPLATES_PATH=/app/services/web/templates \
    SYSTEMPROMPT_ASSETS_PATH=/app/services/web/assets

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

CMD ["/app/bin/systemprompt", "infra", "services", "serve", "--foreground"]
