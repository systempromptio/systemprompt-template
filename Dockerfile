# syntax=docker/dockerfile:1.7
# Multi-stage build for systemprompt-template.
# Stage 1 compiles the Rust workspace against the repo's .sqlx/ offline cache.
# Stage 2 ships a slim Debian runtime with the binaries + services/ YAML tree.

FROM rust:1-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev \
    libssl-dev \
    pkg-config \
    clang \
    mold \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . /src

ENV SQLX_OFFLINE=true \
    CC=clang \
    CXX=clang++

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --workspace \
    && mkdir -p /out/bin \
    && cp target/release/systemprompt /out/bin/ \
    && cp target/release/systemprompt-mcp-agent /out/bin/ \
    && cp target/release/systemprompt-mcp-marketplace /out/bin/

FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.title="systemprompt" \
      org.opencontainers.image.description="AI governance gateway for Claude, OpenAI, and Gemini — policy, audit, and MCP orchestration" \
      org.opencontainers.image.source="https://github.com/systempromptio/systemprompt-template" \
      org.opencontainers.image.url="https://systemprompt.io" \
      org.opencontainers.image.documentation="https://github.com/systempromptio/systemprompt-template/tree/main/docs" \
      org.opencontainers.image.vendor="systemprompt.io" \
      org.opencontainers.image.licenses="MIT AND BUSL-1.1"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libpq5 \
    libssl3 \
    postgresql-client \
    lsof \
    jq \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 app
WORKDIR /app

RUN mkdir -p /app/bin /app/logs /app/storage /app/web /app/services/profiles/docker

COPY --from=builder /out/bin/ /app/bin/

COPY services /app/services
COPY storage /app/storage
COPY migrations /app/migrations
COPY web /app/web
# MCP manifests live alongside their extension crates; the runtime validator
# globs extensions/mcp/*/manifest.yaml to resolve binary -> manifest.
COPY extensions/mcp /app/extensions/mcp

COPY docker/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh /app/bin/* \
    && chown -R app:app /app

USER app
EXPOSE 8080

ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    PATH="/app/bin:${PATH}" \
    SYSTEMPROMPT_SERVICES_PATH=/app/services \
    SYSTEMPROMPT_MCP_PATH=/app/bin \
    SYSTEMPROMPT_PROFILE=/app/services/profiles/docker/profile.yaml \
    WEB_DIR=/app/web

HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

ENTRYPOINT ["/app/entrypoint.sh"]
