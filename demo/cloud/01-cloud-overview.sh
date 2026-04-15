#!/bin/bash
# DEMO: CLOUD OVERVIEW — Auth status, profiles, deployment info
# Read-only cloud operations only.
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "CLOUD: OVERVIEW" "Authentication status, profiles, deployment info"

subheader "STEP 1: Current Auth Status"
cmd "systemprompt cloud auth whoami"
"$CLI" cloud auth whoami --profile "$PROFILE" 2>&1 | sed 's/^/  /' || info "Not authenticated to cloud."
echo ""

subheader "STEP 2: Cloud Status"
cmd "systemprompt cloud status"
"$CLI" cloud status --profile "$PROFILE" 2>&1 | sed 's/^/  /' || info "No cloud deployment configured."
echo ""

subheader "STEP 3: Available Profiles"
run_cli_indented cloud profile list

info ""
info "Cloud operations (deploy, sync, secrets) are not demoed here"
info "as they modify remote infrastructure. Use 'systemprompt cloud --help'"
info "to explore available commands."

header "CLOUD OVERVIEW DEMO COMPLETE"
