# Clean-room image for testing Claude Code against the systemprompt gateway.
#
# Fresh environment: no prior ~/.claude, no login, no bridge config. Proves the
# from-scratch install path. The gateway runs SEPARATELY on the host; this
# container reaches it over `--network host` (see run.sh).
#
# Base is glibc (bookworm-slim, NOT alpine/musl) because the mounted
# systemprompt-bridge binary is built x86_64-unknown-linux-gnu (glibc).
FROM node:22-bookworm-slim

# curl + jq for the smoke test and reading the loopback secret; ca-certificates
# for TLS to a remote gateway; bsdextrautils for `script` (pty wrapper that keeps
# a persistent interactive `claude` listener alive for the channels test).
RUN apt-get update \
 && apt-get install -y --no-install-recommends curl jq ca-certificates bsdextrautils \
 && rm -rf /var/lib/apt/lists/*

# Install Claude Code globally from npm — the real, public install path.
RUN npm install -g @anthropic-ai/claude-code \
 && claude --version

# Unprivileged user with an empty home (no ~/.claude, no bridge config).
RUN useradd --create-home --shell /bin/bash dev
USER dev
WORKDIR /home/dev

# The bridge binary is mounted read-only at runtime (see run.sh), never baked in.
ENV PATH="/usr/local/bin:${PATH}"

# Throwaway custom-channel harness (Stage 1 spike + Stage 2 A->B demo). Built on
# the MCP TypeScript SDK, installed locally so `node webhook-channel.mjs` resolves
# it regardless of cwd. See channels/run-channels-test.sh.
COPY --chown=dev:dev channels /home/dev/channels
RUN cd /home/dev/channels && npm install --omit=dev && chmod +x *.sh *.mjs

COPY --chown=dev:dev entrypoint.sh /home/dev/entrypoint.sh
RUN chmod +x /home/dev/entrypoint.sh

ENTRYPOINT ["/home/dev/entrypoint.sh"]
