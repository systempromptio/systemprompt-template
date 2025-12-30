# SystemPrompt Template - Production Image
# Build locally first: just build --release && just build-mcp
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libpq5 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 app
WORKDIR /app

RUN mkdir -p /app/bin /app/bin/mcp /app/data /app/logs /app/storage

# Copy pre-built Rust binaries
COPY target/release/systemprompt /app/bin/

# Copy pre-built MCP server binaries
COPY target/release/systemprompt-infrastructure /app/bin/mcp/
COPY target/release/systemprompt-admin /app/bin/mcp/
COPY target/release/system-tools /app/bin/mcp/

# Copy services configuration
COPY services /app/services

# Copy profiles
COPY .systemprompt/profiles /app/services/profiles

# Copy entrypoint
COPY .systemprompt/entrypoint.sh /app/entrypoint.sh

# Copy pre-built React frontend
COPY core/web/dist /app/web

# Copy core crates (needed for migrations)
COPY core/crates /app/core/crates

RUN chmod +x /app/bin/* /app/bin/mcp/* /app/entrypoint.sh && chown -R app:app /app

USER app
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    PATH="/app/bin:/app/bin/mcp:$PATH" \
    SYSTEMPROMPT_CORE_PATH=/app/core \
    SYSTEMPROMPT_SERVICES_PATH=/app/services \
    SYSTEMPROMPT_MCP_PATH=/app/bin/mcp \
    WEB_DIR=/app/web

CMD ["/app/entrypoint.sh"]
