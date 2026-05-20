# syntax=docker/dockerfile:1.7
# Mock inference endpoint for the air-gapped deployment scenario.
#
# Build context is the sibling systemprompt-core repo (see
# docker-compose.airgap.yml -> services.mock-inference.build.context), because
# the mock crate lives there and must not be copied into the template repo.
# The crate at crates/tests/mock-inference/ declares its own [workspace], so it
# builds standalone without the rest of the core workspace.

FROM rust:1-bookworm AS builder

WORKDIR /src
COPY crates/tests/mock-inference /src/crates/tests/mock-inference

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/crates/tests/mock-inference/target \
    cargo build --release \
        --manifest-path crates/tests/mock-inference/Cargo.toml \
    && cp crates/tests/mock-inference/target/release/mock-inference /usr/local/bin/mock-inference

FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.title="systemprompt-mock-inference" \
      org.opencontainers.image.description="Deterministic mock inference endpoint for the air-gapped deployment scenario" \
      org.opencontainers.image.vendor="systemprompt.io"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/mock-inference /usr/local/bin/mock-inference

EXPOSE 8080

# The binary always binds 0.0.0.0; --port selects the listen port.
CMD ["mock-inference", "--port", "8080"]
