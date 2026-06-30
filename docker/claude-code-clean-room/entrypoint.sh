#!/usr/bin/env bash
# Clean-room entrypoint: bring up the bridge loopback proxy, wire Claude Code at
# it, run a smoke test, then drop to an interactive shell.
#
# Requires (passed by run.sh): SP_BRIDGE_PAT, SP_BRIDGE_GATEWAY_URL, and the
# systemprompt-bridge binary mounted at /usr/local/bin/systemprompt-bridge.
set -euo pipefail

: "${SP_BRIDGE_GATEWAY_URL:?SP_BRIDGE_GATEWAY_URL is required}"
: "${SP_BRIDGE_PAT:?SP_BRIDGE_PAT is required}"

PROXY_BASE_URL="http://127.0.0.1:48217"
SECRET_FILE="${HOME}/.config/systemprompt/bridge-loopback.key"

echo "==> Clean-room check: nothing pre-installed"
echo "    claude:  $(command -v claude)  ($(claude --version))"
echo "    bridge:  $(command -v systemprompt-bridge)"
echo "    ~/.claude exists? $([ -e "${HOME}/.claude" ] && echo yes || echo no)"
echo "    gateway: ${SP_BRIDGE_GATEWAY_URL}"
echo

echo "==> Starting the bridge loopback proxy (headless)"
systemprompt-bridge proxy >/tmp/bridge-proxy.log 2>&1 &
PROXY_PID=$!
trap 'kill "${PROXY_PID}" 2>/dev/null || true' EXIT

# Wait for the proxy to mint its loopback secret and bind the port.
for _ in $(seq 1 30); do
  [ -s "${SECRET_FILE}" ] && break
  if ! kill -0 "${PROXY_PID}" 2>/dev/null; then
    echo "!! proxy exited early:" >&2; cat /tmp/bridge-proxy.log >&2; exit 1
  fi
  sleep 0.5
done
if [ ! -s "${SECRET_FILE}" ]; then
  echo "!! proxy did not produce a loopback secret in time:" >&2
  cat /tmp/bridge-proxy.log >&2; exit 1
fi
LOOPBACK_SECRET="$(cat "${SECRET_FILE}")"
echo "    proxy up on ${PROXY_BASE_URL} (pid ${PROXY_PID})"
echo

echo "==> Wiring Claude Code at the proxy (settings.json)"
mkdir -p "${HOME}/.claude"
cat > "${HOME}/.claude/settings.json" <<JSON
{
  "env": {
    "ANTHROPIC_BASE_URL": "${PROXY_BASE_URL}",
    "ANTHROPIC_AUTH_TOKEN": "${LOOPBACK_SECRET}"
  }
}
JSON
# Skip the first-run onboarding / trust UI so `claude -p` is non-interactive.
cat > "${HOME}/.claude.json" <<JSON
{ "hasCompletedOnboarding": true, "bypassPermissionsModeAccepted": true }
JSON
# Export for the interactive shell + the smoke test below.
export ANTHROPIC_BASE_URL="${PROXY_BASE_URL}"
export ANTHROPIC_AUTH_TOKEN="${LOOPBACK_SECRET}"
echo "    ANTHROPIC_BASE_URL=${ANTHROPIC_BASE_URL}"
echo

# Model Claude Code will send. claude-* routes to Gemini at the gateway
# (claude-opus-* is the exception → cerebras), so a Sonnet model exercises Gemini.
CC_MODEL="${CC_MODEL:-claude-sonnet-4-6}"

echo "==> Smoke test: raw /v1/messages through the proxy (model ${CC_MODEL})"
curl -fsS -m 40 -X POST "${PROXY_BASE_URL}/v1/messages" \
  -H "Authorization: Bearer ${LOOPBACK_SECRET}" \
  -H "anthropic-version: 2023-06-01" -H "content-type: application/json" \
  -d "{\"model\":\"${CC_MODEL}\",\"max_tokens\":64,\"messages\":[{\"role\":\"user\",\"content\":\"reply with exactly: pong\"}]}" \
  | jq -r '.model as $m | "upstream=\($m) -> \(.content[0].text // .)"' || echo "(smoke test failed — check the gateway)"
echo

echo "==> Claude Code headless run (no login; --model ${CC_MODEL})"
if timeout 90 claude -p --model "${CC_MODEL}" "Reply with exactly one word: pong" </dev/null 2>/tmp/claude-run.err; then
  echo
else
  echo "!! claude -p failed (exit $?); stderr:" >&2; cat /tmp/claude-run.err >&2
fi
echo

echo "==> Installing the org marketplace via the bridge (same signed-manifest sync as the Win/Mac GUI)"
# Pin the manifest signing pubkey (out-of-band on Win/Mac via MDM; here we fetch
# it from the gateway's unauthenticated endpoint and pin it).
PUBKEY="$(curl -fsS "${SP_BRIDGE_GATEWAY_URL}/v1/bridge/pubkey" | jq -r .pubkey)"
echo "    pinned pubkey: ${PUBKEY}"
systemprompt-bridge install --gateway "${SP_BRIDGE_GATEWAY_URL}" --pubkey "${PUBKEY}" 2>&1 | sed 's/^/    /' || true
systemprompt-bridge sync 2>&1 | sed 's/^/    /' || echo "    (sync reported an error)"
echo

echo "==> Verifying installed Claude Code marketplace config (MCP server + skill)"
MP="${HOME}/.claude/plugins/marketplaces/org-provisioned"
BUNDLE="${MP}/plugins/systemprompt-managed"
MCP_FILE="$(find "${BUNDLE}" -name '.mcp.json' 2>/dev/null | head -1)"
SKILL_FILE="$(find "${BUNDLE}" -path '*/skills/*/SKILL.md' 2>/dev/null | head -1)"
ORG_PLUGINS="${XDG_DATA_HOME:-${HOME}/.local/share}/Claude/org-plugins/systemprompt-managed"
fail=0
ck() { if eval "$2" >/dev/null 2>&1; then echo "    ✓ $1"; else echo "    ✗ $1"; fail=1; fi; }
ck "marketplace.json written"                 "[ -f '${MP}/.claude-plugin/marketplace.json' ]"
ck "plugin.json written"                      "[ -f '${BUNDLE}/.claude-plugin/plugin.json' ]"
ck ".mcp.json present"                         "[ -n '${MCP_FILE}' ]"
ck ".mcp.json registers 'systemprompt' MCP"   "jq -e '.mcpServers.systemprompt' '${MCP_FILE}'"
ck "skill SKILL.md installed"                  "[ -n '${SKILL_FILE}' ]"
ck "installed_plugins.json records plugin"    "jq -e '.plugins[\"systemprompt-managed@org-provisioned\"]' '${HOME}/.claude/plugins/installed_plugins.json'"
ck "known_marketplaces.json records org"      "jq -e '.[\"org-provisioned\"]' '${HOME}/.claude/plugins/known_marketplaces.json'"
ck "settings.json enables the plugin"         "jq -e '.enabledPlugins[\"systemprompt-managed@org-provisioned\"]==true' '${HOME}/.claude/settings.json'"
ck "settings.json env block preserved"        "jq -e '.env.ANTHROPIC_BASE_URL' '${HOME}/.claude/settings.json'"
ck "synthetic org-plugins bundle written"     "[ -d '${ORG_PLUGINS}' ]"
echo
echo "    MCP servers in .mcp.json:   $(jq -rc '.mcpServers | keys' "${MCP_FILE}" 2>/dev/null)"
echo "    systemprompt MCP url:       $(jq -r '.mcpServers.systemprompt.url' "${MCP_FILE}" 2>/dev/null)"
echo "    skills installed:           $(find "${BUNDLE}/skills" -maxdepth 1 -mindepth 1 -type d -printf '%f ' 2>/dev/null)"
echo "    enabled plugins (settings): $(jq -rc '.enabledPlugins' "${HOME}/.claude/settings.json" 2>/dev/null)"
echo
echo "    --- Claude Code's own view (claude plugin / mcp) ---"
timeout 30 claude plugin list </dev/null 2>&1 | sed 's/^/    /' | head -20 || true
timeout 30 claude mcp list </dev/null 2>&1 | sed 's/^/    /' | head -20 || true
echo
if [ "${fail}" -eq 0 ]; then
  echo "    MARKETPLACE INSTALL: all checks passed ✓"
else
  echo "    MARKETPLACE INSTALL: one or more checks FAILED ✗"
fi
echo

echo "==> Ready. The proxy is running; ANTHROPIC_* are exported."
echo "    Try:  claude -p \"what model are you?\""
echo "    Or just:  claude"
# Only drop to an interactive shell when attached to a TTY; otherwise exit so
# automated runs (CI / docker run without -it) terminate cleanly.
if [ -t 0 ]; then
  exec bash
fi
echo "==> Non-interactive run complete."
