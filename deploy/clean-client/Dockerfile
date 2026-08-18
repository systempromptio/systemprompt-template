# syntax=docker/dockerfile:1.7
#
# A clean Linux client for testing the Claude Code + Astound Bridge integration.
#
# This image deliberately contains NO Astound config. It exists to prove the
# integration works from a machine that has never been set up: bridge login,
# bridge install, credential helper, plugin/MCP sync, then a real Claude Code
# session routed through the gateway.
#
# The build context is this directory only — nothing from the repo is copied in,
# so the image cannot inherit host state. The bridge binary is bind-mounted at
# run time by `just clean-client` instead of being baked in, so you always test
# the binary you just built.

FROM node:22-trixie-slim

# ripgrep and less are used by Claude Code itself (search + pager); git because
# most of what you will do in the session is in a repo; jq for poking at the
# bridge's JSON output during debugging.
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates \
      curl \
      git \
      jq \
      less \
      openssl \
      procps \
      ripgrep \
    && rm -rf /var/lib/apt/lists/*

# The bridge dynamically links libdbus-1 on Linux, via keyring-core's
# dbus-secret-service-keyring-store — without it the binary will not even print
# its help ("error while loading shared libraries: libdbus-1.so.3"). `dbus`
# itself provides dbus-run-session, needed because keyring writes require a
# session bus and a container has none:
#   dbus-run-session -- astound-bridge <cmd>
# Any Linux tarball we ship has the same runtime dependency — see README.
RUN apt-get update && apt-get install -y --no-install-recommends \
      dbus \
      libdbus-1-3 \
    && rm -rf /var/lib/apt/lists/*

# Playwright + Chromium, so the dev_test skill ("no change is done until
# Playwright proves it") works out of the box against a mounted project.
# INSTALL_BROWSERS=0 gives the slim bridge-integration-only variant — the
# browser layer is ~1GB. Browsers live in a shared path readable by the
# unprivileged tester user created below.
ARG INSTALL_BROWSERS=1
ENV PLAYWRIGHT_BROWSERS_PATH=/opt/ms-playwright
RUN if [ "$INSTALL_BROWSERS" = "1" ]; then \
      npm install -g playwright@latest \
      && playwright install --with-deps chromium \
      && chmod -R a+rX /opt/ms-playwright \
      && npm cache clean --force \
      && rm -rf /var/lib/apt/lists/*; \
    fi

# uid 1001 because the node base image already owns 1000. The whole point of
# this container is a virgin HOME, so the user is created empty and never has
# anything copied into it.
RUN useradd --create-home --shell /bin/bash --uid 1001 tester

# sudo, so the enforced managed-MCP path is reachable. Claude Code reads its
# enterprise MCP policy from /etc/claude-code, which an unprivileged installer
# cannot write — on a real fleet that directory is provisioned by MDM, not by
# the user. NOPASSWD lets a test run stand in for that provisioning step
# explicitly (`sudo install -d -o tester /etc/claude-code`) rather than by
# running the whole bridge as root, which would move its config to /root and
# stop testing the unprivileged layout that a real workstation uses.
RUN apt-get update && apt-get install -y --no-install-recommends sudo \
    && rm -rf /var/lib/apt/lists/* \
    && echo 'tester ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/tester \
    && chmod 0440 /etc/sudoers.d/tester

COPY --chmod=0755 entrypoint.sh /usr/local/bin/entrypoint.sh

# Claude Code installs as tester, into a tester-owned npm prefix, because its
# auto-updater writes the new version into that prefix at run time. Installed as
# root it lands in a root-owned prefix and every session opens with "Auto-update
# failed: no write permission to npm prefix".
#
# The prefix is /opt, not $HOME: /home/tester is a named volume at run time
# (`just claude`), and a volume that already exists shadows whatever the image
# put there — so a HOME install would silently disappear on the second run.
#
# /etc/profile.d because the session is entered through a login shell
# (`bash -lc claude`), and /etc/profile resets PATH unconditionally, dropping
# the ENV set below.
ENV NPM_CONFIG_PREFIX=/opt/npm-global \
    PATH=/opt/npm-global/bin:$PATH
RUN install -d -o tester -g tester /opt/npm-global \
    && printf 'export NPM_CONFIG_PREFIX=/opt/npm-global\nexport PATH=/opt/npm-global/bin:$PATH\n' \
       > /etc/profile.d/npm-global.sh

USER tester

RUN npm install -g @anthropic-ai/claude-code@latest \
    && npm cache clean --force

WORKDIR /home/tester/work

# ASTOUND_BRIDGE_GATEWAY_URL is the one setting that must point back out of the
# container; `just clean-client` overrides it. host.docker.internal resolves to
# the WSL host via --add-host, so a gateway running in your primary distro on
# :8080 is reachable without publishing anything.
ENV ASTOUND_BRIDGE_GATEWAY_URL=http://host.docker.internal:8080 \
    TERM=xterm-256color

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["bash", "-l"]
