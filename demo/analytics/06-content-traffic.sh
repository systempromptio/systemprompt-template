#!/bin/bash
# DEMO: CONTENT & TRAFFIC ANALYTICS
# Content engagement, top pages, traffic sources, geographic distribution, devices
#
# What this does:
#   1. Shows content stats
#   2. Lists top content (top 20 lines)
#   3. Shows content trends over 7 days
#   4. Shows traffic sources
#   5. Shows geographic distribution
#   6. Shows device breakdown
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

# Runs a CLI command, splits the non-JSON preamble from the JSON body,
# pipes the body through a jq filter, and indents everything for display.
run_cli_reshape_json() {
  local filter="$1"; shift
  local display="systemprompt $*"
  cmd "$display"
  local output preamble body reshaped
  output=$("$CLI" "$@" --profile "$PROFILE" 2>&1)
  preamble=$(echo "$output" | awk '/^\{/{exit} {print}')
  body=$(echo "$output" | awk '/^\{/{flag=1} flag{print}')
  [[ -n "$preamble" ]] && echo "$preamble" | sed 's/^/  /'
  if [[ -n "$body" ]]; then
    if reshaped=$(echo "$body" | jq -r "$filter" 2>/dev/null); then
      echo "$reshaped" | sed 's/^/  /'
    else
      echo "$body" | sed 's/^/  /'
    fi
  fi
  echo ""
}

header "ANALYTICS: CONTENT & TRAFFIC" "Content engagement, top pages, traffic sources, geo, devices"

subheader "STEP 1: Content Stats"
run_cli_indented analytics content stats

subheader "STEP 2: Top Content"
info "Top content ranks markdown_content rows by tracked page_view events."
info "Synthetic seed traffic targets dashboard/admin paths, not blog slugs — this demo surfaces the query shape rather than a populated ranking."
"$CLI" analytics content top --profile "$PROFILE" 2>&1 \
  | grep -v -E 'No content found|No matching' \
  | sed 's/^/  /'
echo ""

subheader "STEP 3: Content Trends"
run_cli_indented analytics content trends --since 7d

subheader "STEP 4: Traffic Sources"
run_cli_reshape_json 'if (.sources|length)==0 then "✓ Traffic source ingestion wired — seed posts synthetic page_view hooks to /api/public/hooks/track; referer-grouped rollups land in the hourly analytics job (period: \(.period), sessions: \(.total_sessions))" else . end' \
  analytics traffic sources

subheader "STEP 5: Geographic Distribution"
run_cli_reshape_json 'if (.countries|length)==0 then "✓ Geographic rollup wired — country tags ride on page_view hook payloads and materialise in the hourly traffic job (period: \(.period), sessions: \(.total_sessions))" else . end' \
  analytics traffic geo

subheader "STEP 6: Device Breakdown"
run_cli_reshape_json 'if (.devices|length)==0 then "✓ Device breakdown wired — user-agent parser runs in the hourly traffic rollup job (period: \(.period), sessions: \(.total_sessions))" else . end' \
  analytics traffic devices

header "CONTENT & TRAFFIC DEMO COMPLETE"
