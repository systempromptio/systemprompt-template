# Stage 1: Build web assets
FROM node:20-bookworm-slim AS web-builder

WORKDIR /build/core/web
COPY core/web/package*.json core/web/.npmrc ./
RUN npm ci
COPY core/web ./
RUN npm run build

# Stage 2: Build Rust binary with Postgres for sqlx
FROM rust:1.83-bookworm AS rust-builder

# Install postgres and build deps
RUN apt-get update && apt-get install -y \
    libpq-dev \
    postgresql \
    postgresql-contrib \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy source
COPY core ./core
COPY extensions ./extensions
COPY Cargo.toml ./
COPY src ./src
COPY build.rs ./

# Start postgres, create database, then build
RUN service postgresql start && \
    su - postgres -c "psql -c \"CREATE USER systemprompt WITH PASSWORD 'systemprompt' SUPERUSER;\"" && \
    su - postgres -c "createdb -O systemprompt systemprompt" && \
    DATABASE_URL="postgres://systemprompt:systemprompt@localhost/systemprompt" \
    cargo build --release --manifest-path=core/Cargo.toml --target-dir=target

# Stage 3: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libpq5 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 app
WORKDIR /app

RUN mkdir -p /app/bin /app/data /app/logs /app/storage

# Copy binary from rust builder
COPY --from=rust-builder /build/target/release/systemprompt /app/bin/

# Copy web assets from web builder
COPY --from=web-builder /build/core/web/dist /app/web

# Copy service configuration
COPY services /app/services

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
