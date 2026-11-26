# syntax=docker/dockerfile:1.7
# One image per flavour (flavours/<name>/flavour.yaml). Build through
# scripts/flavour-build.sh, which reads the flavour and passes these arguments.
# Core resolves from the git-rev pin in Cargo.toml: cargo fetches it into the
# cached ~/.cargo/git (git and CA certificates ship in the rust image). Only
# while a local core override is on (scripts/core-patch.sh) does the script
# pass that checkout as the `core` build context, mounted at CORE_MOUNT. The
# defaults build `full`.
#   builder    the flavour's binaries: dependencies first, then sources
#   services   the flavour's services tree (scripts/flavour-stage-services.sh)
#   artifacts  the binaries alone: --target artifacts --output type=local,dest=out
#   cache-reset  empties the flavour's target cache (clean-build metrics)
#   runtime    runtime-slim (distroless plus what the entrypoint and
#              healthcheck need) or runtime-full (adds demo/ and hey), chosen
#              by RUNTIME
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
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=target-${FLAVOUR},target=/src/target,sharing=locked \
    --mount=type=bind,from=core,target=${CORE_MOUNT} \
    cargo build --release --locked --no-default-features --features "$FEATURES" \
        $(printf -- '--bin %s ' $BINS)
COPY . /src
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git,sharing=locked \
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

# The flavour's content: its services tree (scripts/flavour-stage-services.sh),
# the storage tree it serves and its profile base layer. Storage the flavour
# links no member for is left out: the site's video assets without the public
# site (SITE is not true), the hosted plugin bundles without a staged
# services/plugins tree. The base layer is flavours/profile.base.yaml with the
# flavour's profile.overlay.yaml merged over it the way core merges layers
# (maps key by key, anything else replaced whole); core reads it as
# <profiles>/base.yaml beneath the profile admin setup authors.
FROM debian:bookworm-slim AS services
ARG FLAVOUR
ARG SITE
RUN apt-get update && apt-get install -y --no-install-recommends python3 python3-yaml \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /stage
COPY flavours /stage/flavours
COPY services /stage/services
COPY scripts/flavour.py scripts/flavour-stage-services.sh /stage/scripts/
COPY scripts/lib /stage/scripts/lib
COPY storage /staged/storage
RUN bash scripts/flavour-stage-services.sh "$FLAVOUR" /staged/services \
    && if [ "$SITE" != true ]; then rm -rf /staged/storage/files/video; fi \
    && if [ ! -d /staged/services/plugins ]; then rm -rf /staged/storage/files/plugins; fi
RUN python3 - flavours/profile.base.yaml "flavours/$FLAVOUR/profile.overlay.yaml" > /staged/profile.base.yaml <<'PY'
import sys, yaml
def merge(base, overlay):
    if isinstance(base, dict) and isinstance(overlay, dict):
        for key, value in overlay.items():
            base[key] = merge(base[key], value) if key in base else value
        return base
    return overlay
layers = [yaml.safe_load(open(path)) or {} for path in sys.argv[1:]]
merged = {}
for layer in layers:
    merged = merge(merged, layer)
yaml.safe_dump(merged, sys.stdout, sort_keys=False)
PY

# hey drives the demo/performance load tests; its upstream binary host answers
# 403, so it is built from source.
FROM golang:1-bookworm AS heybuilder
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go install github.com/rakyll/hey@latest

# The runtime base: glibc, CA certificates and tzdata, no shell, no package
# manager and no OpenSSL. The binary links libc, libm and libgcc_s only
# (runtime-tools adds libgcc_s); outbound TLS reads the CA bundle.
FROM gcr.io/distroless/base-nossl-debian12 AS distroless

# What the entrypoint and the healthcheck run, taken from the same Debian
# release as the base (docker/runtime-tools.sh lists each tool's reason).
FROM debian:bookworm-slim AS runtime-tools
RUN apt-get update && apt-get install -y --no-install-recommends busybox jq lsof \
    && rm -rf /var/lib/apt/lists/*
COPY --from=distroless / /distroless
COPY --from=builder /out/bin /flavour-bin
COPY docker/runtime-tools.sh /runtime-tools.sh
RUN sh /runtime-tools.sh /distroless /rootfs /apphome /flavour-bin

FROM distroless AS runtime-slim
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
# /app and /home/app come owned by the app account (uid 1000): a named volume
# mounted at a path the image lacks comes up root-owned, and the server cannot
# write it. Ownership is set on COPY: a recursive chown afterwards would store
# every copied file, the binary included, a second time in its own layer.
COPY --from=runtime-tools /rootfs/ /
COPY --from=runtime-tools --chown=1000:1000 /apphome/ /
WORKDIR /app
COPY --from=builder --chown=1000:1000 --chmod=0755 /out/bin/ /app/bin/
COPY --from=services --chown=1000:1000 /staged/services /app/services
COPY --from=services --chown=1000:1000 /staged/storage /app/storage
COPY --from=services --chown=1000:1000 /staged/profile.base.yaml /app/profile.base.yaml
COPY --chown=1000:1000 web /app/web
COPY --chown=1000:1000 --chmod=0755 docker/entrypoint.sh docker/healthcheck.sh /app/
USER app
EXPOSE 8080
ENV HOME=/home/app \
    HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    PATH="/app/bin:/usr/local/bin:/usr/bin:/bin" \
    SYSTEMPROMPT_SERVICES_PATH=/app/services \
    SYSTEMPROMPT_MCP_PATH=/app/bin \
    SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/docker/profile.yaml \
    WEB_DIR=/app/web \
    SYSTEMPROMPT_FLAVOUR="${FLAVOUR}" \
    FLAVOUR_REQUIRES_AI_KEY="${REQUIRES_AI_KEY}" \
    FLAVOUR_SITE="${SITE}"
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD ["/app/healthcheck.sh"]
ENTRYPOINT ["/app/entrypoint.sh"]

FROM runtime-slim AS runtime-full
COPY --from=heybuilder --chmod=0755 /go/bin/hey /app/bin/hey
# The homepage demo showcase scans <system_root>/demo at runtime.
COPY --chown=app:app demo /app/demo

FROM runtime-${RUNTIME} AS runtime
