#!/bin/bash
# DEMO: WEB CONFIGURATION — Content types, templates, assets
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

header "WEB: CONFIGURATION" "Content types, templates, assets"

subheader "STEP 1: Content Types"
run_cli_reshape_json 'if (.content_types|length)==0 then "✓ No custom content types registered — content kinds come from services/content/config.yaml allowed_content_types (blog, guide, tutorial, reference, docs)" else . end' \
  web content-types list

subheader "STEP 2: Templates"
run_cli_reshape_json 'if (.templates|length)==0 then "✓ No template overrides — handlebars templates load from storage/files/admin/templates/ at SSR time" else . end' \
  web templates list

subheader "STEP 3: Assets"
run_cli_reshape_json 'if (.assets|length)==0 then "✓ No registered dynamic assets — web assets served from web/dist/ after just publish" else . end' \
  web assets list

header "WEB CONFIG DEMO COMPLETE"
