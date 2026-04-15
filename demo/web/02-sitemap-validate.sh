#!/bin/bash
# DEMO: SITEMAP & VALIDATION — Sitemap config, web validation
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

# Runs a CLI command, splits the non-JSON preamble from the JSON body,
# pipes the body through a jq filter to reshape intrinsically-empty
# fields into prose, and indents everything for display.
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

header "WEB: SITEMAP & VALIDATION" "Sitemap config, web validation"

subheader "STEP 1: Sitemap Configuration"
run_cli_reshape_json 'if (.routes|length)==0 then "✓ Sitemap registered — \(.total_routes) static route(s); dynamic blog/documentation routes resolve from content sources at render time" else . end' \
  web sitemap show

subheader "STEP 2: Validate Web Configuration"
run_cli_reshape_json 'if .valid and (.errors|length)==0
                      then "✓ Valid — \(.items_checked) items checked, no errors (\((.warnings//[])|length) warning(s))"
                      + (if ((.warnings//[])|length)>0
                         then "\n" + ((.warnings|map("  - [\(.source)] \(.message)"))|join("\n"))
                         else "" end)
                      else .
                      end' \
  web validate

header "WEB VALIDATION DEMO COMPLETE"
