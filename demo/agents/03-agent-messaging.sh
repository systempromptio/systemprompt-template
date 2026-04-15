#!/bin/bash
# DEMO: AGENT TRACING — Full Pipeline with Artifacts & MCP
#
# Messages an agent, captures its MCP tool use, artifact, and execution
# trace. Requires at least one agent configured in services/agents/. If
# none exist (the default template state), the demo prints an explanation
# and exits cleanly.
#
# Cost: ~$0.01 (one AI call) when agents are configured, else free.

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

PROFILE="${1:-local}"

header "DEMO: AGENT TRACING" "Full Pipeline with Artifacts & MCP"

# ──────────────────────────────────────────────
#  Preflight: is any agent configured?
# ──────────────────────────────────────────────
LIST_OUTPUT=$("$CLI" admin agents list --profile "$PROFILE" 2>&1)
TARGET_AGENT=$(echo "$LIST_OUTPUT" | grep -oP '"(name|id)":\s*"\K[^"]+' | head -1 || true)

if [[ -z "$TARGET_AGENT" ]]; then
  info "No agents configured in services/agents/ (empty template state)."
  info ""
  info "This demo exercises the platform agent runtime — messaging, AI"
  info "reasoning, MCP tool use, artifacts, and tracing. It requires at"
  info "least one agent YAML at services/agents/<id>.yaml and the id"
  info "listed in a plugin under services/plugins/<plugin>.yaml."
  info ""
  info "To enable this demo, add an agent YAML and run: just start"
  header "DEMO SKIPPED — no agents configured"
  exit 0
fi

echo "  Target agent: $TARGET_AGENT"
echo ""

# ──────────────────────────────────────────────
#  STEP 1: Create a context
# ──────────────────────────────────────────────
subheader "STEP 1: Create a context"

CONTEXT_NAME="Demo agent tracing $(date +%H:%M:%S)"
echo "  \$ systemprompt core contexts create --name \"$CONTEXT_NAME\""
echo ""

CONTEXT_OUTPUT=$("$CLI" core contexts create --name "$CONTEXT_NAME" --profile "$PROFILE" 2>&1)
echo "$CONTEXT_OUTPUT" | sed 's/^/  /'
echo ""

CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep -oP '"id":\s*"\K[^"]+' | head -1 || true)
if [[ -z "$CONTEXT_ID" ]]; then
  CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep -oP '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}' | head -1 || true)
fi

if [[ -z "$CONTEXT_ID" ]]; then
  echo "  ERROR: Could not extract context ID from output" >&2
  exit 1
fi

echo "  Context ID: $CONTEXT_ID"
echo ""

# ──────────────────────────────────────────────
#  STEP 2: Message the agent
# ──────────────────────────────────────────────
subheader "STEP 2: Message $TARGET_AGENT" "Asking it to list all agents on the platform"

echo "  \$ systemprompt admin agents message $TARGET_AGENT \\"
echo "      -m \"List all agents running on this platform\" \\"
echo "      --context-id \"$CONTEXT_ID\" --blocking --timeout 60"
echo ""

set +e
MESSAGE_OUTPUT=$("$CLI" admin agents message "$TARGET_AGENT" \
  -m "List all agents running on this platform" \
  --context-id "$CONTEXT_ID" \
  --blocking --timeout 60 \
  --profile "$PROFILE" 2>&1)
MESSAGE_RC=$?
set -e

echo "  Agent response (truncated):"
echo "$MESSAGE_OUTPUT" | head -40 | sed 's/^/  /'
LINES=$(echo "$MESSAGE_OUTPUT" | wc -l)
if [[ "$LINES" -gt 40 ]]; then
  echo "  ... ($((LINES - 40)) more lines)"
fi
echo ""

if [[ "$MESSAGE_RC" -ne 0 ]] \
   || echo "$MESSAGE_OUTPUT" | grep -qiE "API key not valid|API_KEY_INVALID|Failed to send message|Gemini API error|Agent returned error|Internal error"; then
  echo "  ERROR: agent conversation failed." >&2
  if echo "$MESSAGE_OUTPUT" | grep -qiE "API key not valid|API_KEY_INVALID"; then
    echo "  Cause: Gemini API key is invalid or missing." >&2
    echo "  Fix:   set a valid key in .systemprompt/profiles/$PROFILE/secrets.json" >&2
    echo "         under the \"gemini\" field, then restart services (just start)." >&2
  fi
  exit 1
fi

# ──────────────────────────────────────────────
#  STEP 3: Retrieve artifact
# ──────────────────────────────────────────────
subheader "STEP 3: Retrieve artifact"

echo "  \$ systemprompt core artifacts list --context-id \"$CONTEXT_ID\""
echo ""

ARTIFACTS_OUTPUT=$("$CLI" core artifacts list --context-id "$CONTEXT_ID" --profile "$PROFILE" 2>&1)
echo "$ARTIFACTS_OUTPUT" | head -20 | sed 's/^/  /'
echo ""

ARTIFACT_ID=$(echo "$ARTIFACTS_OUTPUT" | grep -oP '"id":\s*"\K[^"]+' | head -1 || true)
if [[ -z "$ARTIFACT_ID" ]]; then
  ARTIFACT_ID=$(echo "$ARTIFACTS_OUTPUT" | grep -oP '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}' | head -1 || true)
fi

if [[ -n "$ARTIFACT_ID" ]]; then
  echo "  Artifact ID: $ARTIFACT_ID"
  echo ""
  "$CLI" core artifacts show "$ARTIFACT_ID" --full --profile "$PROFILE" 2>&1 | head -50 | sed 's/^/  /'
  echo ""
fi

# ──────────────────────────────────────────────
#  STEP 4: Execution trace
# ──────────────────────────────────────────────
subheader "STEP 4: Execution trace"

TRACE_OUTPUT=$("$CLI" infra logs trace list --limit 3 --profile "$PROFILE" 2>&1)
echo "$TRACE_OUTPUT" | head -20 | sed 's/^/  /'
echo ""

TRACE_ID=$(echo "$TRACE_OUTPUT" | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1 || true)
if [[ -n "$TRACE_ID" ]]; then
  echo "  Trace ID: $TRACE_ID"
  echo ""
  "$CLI" infra logs trace show "$TRACE_ID" --all --profile "$PROFILE" 2>&1 | head -60 | sed 's/^/  /'
fi

# ──────────────────────────────────────────────
#  STEP 5: Cost breakdown
# ──────────────────────────────────────────────
subheader "STEP 5: Cost breakdown by agent"
"$CLI" analytics costs breakdown --by agent --profile "$PROFILE" 2>&1 | head -30 | sed 's/^/  /'
echo ""

header "AGENT TRACING DEMO COMPLETE"
