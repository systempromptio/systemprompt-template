#!/bin/bash
# DEMO: WEB CONFIGURATION — Content types, templates, assets
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

# Show a web config listing (box table) and assert it is non-empty in the
# structured --json output, failing loudly if the registry is unexpectedly bare.
web_list() {
  local sub="$1" label="$2"
  run_cli_indented web $sub
  assert_min "$(cli_json web $sub | jq '.items | length')" 1 "$label"
}

header "WEB: CONFIGURATION" "Content types, templates, assets"

subheader "STEP 1: Content Types"
web_list "content-types list" "content types registered (blog, documentation)"

subheader "STEP 2: Templates"
web_list "templates list" "page templates registered"

subheader "STEP 3: Assets"
web_list "assets list" "web assets registered"

header "WEB CONFIG DEMO COMPLETE"
