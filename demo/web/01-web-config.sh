#!/bin/bash
# DEMO: WEB CONFIGURATION — Content types, templates, assets
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "WEB: CONFIGURATION" "Content types, templates, assets"

subheader "STEP 1: Content Types"
run_cli_head 20 web content-types list

subheader "STEP 2: Templates"
run_cli_head 20 web templates list

subheader "STEP 3: Assets"
run_cli_head 30 web assets list

header "WEB CONFIG DEMO COMPLETE"
