# SystemPrompt Template - Fly.io Deployment
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libpq5 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 app
WORKDIR /app

# Create directory structure
RUN mkdir -p /app/bin /app/data /app/logs /app/storage

# Copy pre-built binaries (core CLI + any MCP servers)
COPY target/release/systemprompt* /app/bin/

# Copy service configuration
COPY services /app/services

# Copy web assets
COPY core/web/dist /app/web

# Fix permissions
RUN chmod +x /app/bin/* && \
    chown -R app:app /app

USER app

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    PATH="/app/bin:$PATH" \
    SYSTEMPROMPT_SERVICES_PATH=/app/services \
    WEB_DIR=/app/web

CMD ["/app/bin/systemprompt", "services", "serve", "--foreground"]
