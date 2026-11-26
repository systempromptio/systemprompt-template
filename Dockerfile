# syntax=docker/dockerfile:1.7
# One image per flavour (flavours/<name>/flavour.yaml). Build through
# scripts/flavour-build.sh, which reads the flavour and passes these arguments
# (and, while Cargo.toml patches core to a sibling checkout, that checkout as
# the `core` build context). The defaults build `full`.
#   builder    the flavour's binaries: dependencies first, then sources
#   services   the flavour's services tree (scripts/flavour-stage-services.sh)
#   artifacts  the binaries alone: --target artifacts --output type=local,dest=out
#   cache-reset  empties the flavour's target cache (clean-build metrics)
#   runtime    runtime-slim (what the entrypoint and healthcheck need) or
#              runtime-full (adds demo/ and hey), chosen by RUNTIME
ARG FLAVOUR=full
ARG FEATURES=flavour-full
ARG BINS=systemprompt
ARG RUNTIME=full
ARG REQUIRES_AI_KEY=true
ARG SITE=true
ARG CORE_MOUNT=/opt/core-unused

FROM scratch AS core

FROM rust:1-bookworm AS toolchain
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev \
    libssl-dev \
    pkg-config \
    clang \
    mold \
    build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY rust-toolchain.toml /src/rust-toolchain.toml
RUN rustup show
ENV SQLX_OFFLINE=true \
    CC=clang \
    CXX=clang++ \
    RUSTFLAGS="-D warnings"

FROM toolchain AS skeleton
COPY . /context
RUN bash /context/docker/cargo-skeleton.sh /context /skeleton

FROM toolchain AS builder
ARG FLAVOUR
ARG FEATURES
ARG BINS
ARG CORE_MOUNT
COPY --from=skeleton /skeleton /src
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=target-${FLAVOUR},target=/src/target,sharing=locked \
    --mount=type=bind,from=core,target=${CORE_MOUNT} \
    cargo build --release --locked --no-default-features --features "$FEATURES" \
        $(printf -- '--bin %s ' $BINS)
COPY . /src
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=target-${FLAVOUR},target=/src/target,sharing=locked \
    --mount=type=bind,from=core,target=${CORE_MOUNT} \
    find /src/src /src/extensions -name '*.rs' -exec touch {} + \
    && cargo build --release --locked --no-default-features --features "$FEATURES" \
        $(printf -- '--bin %s ' $BINS) \
    && mkdir -p /out/bin \
    && for bin in $BINS; do cp "target/release/$bin" /out/bin/; done

# Empties this flavour's target cache: scripts/flavour-metrics.sh builds this
# before timing a clean compile.
FROM debian:bookworm-slim AS cache-reset
ARG FLAVOUR
RUN --mount=type=cache,id=target-${FLAVOUR},target=/target,sharing=locked \
    find /target -mindepth 1 -delete

FROM scratch AS artifacts
COPY --from=builder /out/bin /bin

FROM debian:bookworm-slim AS services
ARG FLAVOUR
RUN apt-get update && apt-get install -y --no-install-recommends python3 python3-yaml \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /stage
COPY flavours /stage/flavours
COPY services /stage/services
COPY scripts/flavour.py scripts/flavour-stage-services.sh /stage/scripts/
COPY scripts/lib /stage/scripts/lib
RUN bash scripts/flavour-stage-services.sh "$FLAVOUR" /staged/services

# hey drives the demo/performance load tests; its upstream binary host answers
# 403, so it is built from source.
FROM golang:1-bookworm AS heybuilder
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go install github.com/rakyll/hey@latest

# Packages, each for the entrypoint or the healthcheck:
#   ca-certificates  outbound TLS (providers, cloud, OAuth issuers)
#   curl             HEALTHCHECK
#   gosu             drop from root after preparing a root-owned volume
#   jq               secrets.json edits in entrypoint.sh
#   lsof             core's process supervision finds the pids holding a port
#   openssl          signing-key generation in container-state.py
#   postgresql-client  pg_isready before migrating
#   python3, python3-yaml  container-state.py and migrate-profile.py rewrite the
#                    persisted profile YAML; no `admin config` command sets
#                    security.allow_registration, rate_limits.disabled or
#                    security.signing_key_path, and pre-0.47 volumes still
#                    need migrate-profile.py
FROM debian:bookworm-slim AS runtime-slim
ARG FLAVOUR
ARG REQUIRES_AI_KEY
ARG SITE
LABEL org.opencontainers.image.title="systemprompt" \
      org.opencontainers.image.description="AI governance gateway for Claude, OpenAI, and Gemini — policy, audit, and MCP orchestration" \
      org.opencontainers.image.source="https://github.com/systempromptio/systemprompt-template" \
      org.opencontainers.image.url="https://systemprompt.io" \
      org.opencontainers.image.documentation="https://github.com/systempromptio/systemprompt-template/tree/main/docs" \
      org.opencontainers.image.vendor="systemprompt.io" \
      org.opencontainers.image.licenses="MIT AND BUSL-1.1" \
      org.systemprompt.flavour="${FLAVOUR}"
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    gosu \
    jq \
    lsof \
    openssl \
    postgresql-client \
    python3 \
    python3-yaml \
    && rm -rf /var/lib/apt/lists/*
# /app/storage/data is runtime state (excluded from the build context), so it
# is created here: a named volume mounted at a path the image lacks comes up
# root-owned, and the server, running as uid 1000, cannot write it. Ownership
# is set on COPY: a recursive chown afterwards would store every copied file,
# the binary included, a second time in its own layer.
RUN useradd -m -u 1000 app \
    && mkdir -p /app/bin /app/logs /app/data /app/storage/data \
        /app/storage/files/uploads /app/storage/files/images/generated \
        /app/web /app/.systemprompt/profiles/docker \
    && chown -R app:app /app
WORKDIR /app
COPY --from=builder --chown=app:app --chmod=0755 /out/bin/ /app/bin/
COPY --from=services --chown=app:app /staged/services /app/services
COPY --chown=app:app storage /app/storage
COPY --chown=app:app web /app/web
COPY --chown=app:app --chmod=0755 docker/entrypoint.sh /app/entrypoint.sh
COPY --chown=app:app docker/migrate-profile.py docker/container-state.py /app/
COPY --chown=app:app flavours/profile.base.yaml flavours/${FLAVOUR}/profile.overlay.yaml /app/profile-overlay/
USER app
EXPOSE 8080
ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    PATH="/app/bin:${PATH}" \
    SYSTEMPROMPT_SERVICES_PATH=/app/services \
    SYSTEMPROMPT_MCP_PATH=/app/bin \
    SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/docker/profile.yaml \
    WEB_DIR=/app/web \
    SYSTEMPROMPT_FLAVOUR="${FLAVOUR}" \
    FLAVOUR_REQUIRES_AI_KEY="${REQUIRES_AI_KEY}" \
    FLAVOUR_SITE="${SITE}"
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1
ENTRYPOINT ["/app/entrypoint.sh"]

FROM runtime-slim AS runtime-full
COPY --from=heybuilder --chmod=0755 /go/bin/hey /app/bin/hey
# The homepage demo showcase scans <system_root>/demo at runtime.
COPY --chown=app:app demo /app/demo

FROM runtime-${RUNTIME} AS runtime
