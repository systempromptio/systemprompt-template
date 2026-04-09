#!/bin/bash
# DEMO: SITEMAP & VALIDATION — Sitemap config, web validation
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "WEB: SITEMAP & VALIDATION" "Sitemap config, web validation"

subheader "STEP 1: Sitemap Configuration"
run_cli_indented web sitemap show

subheader "STEP 2: Validate Web Configuration"
run_cli_indented web validate

header "WEB VALIDATION DEMO COMPLETE"
