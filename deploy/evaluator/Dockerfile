# syntax=docker/dockerfile:1.7
FROM node:22-trixie-slim
ARG CLAUDE_VERSION=2.1.266
ARG OPENCODE_VERSION=1.18.29
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates git ripgrep \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1001 tester \
    && npm install --global @anthropic-ai/claude-code@${CLAUDE_VERSION} opencode-ai@${OPENCODE_VERSION} \
    && npm cache clean --force \
    && install -d -o tester -g tester /home/tester/work
ENV HOME=/home/tester \
    DISABLE_AUTOUPDATER=1 \
    CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 \
    OPENCODE_DISABLE_AUTOUPDATE=1 \
    XDG_CONFIG_HOME=/home/tester/.config \
    XDG_CACHE_HOME=/home/tester/.cache \
    XDG_DATA_HOME=/home/tester/.local/share
USER tester
WORKDIR /home/tester/work
