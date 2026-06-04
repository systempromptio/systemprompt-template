#!/bin/bash
# DEMO: SITEMAP & VALIDATION — Sitemap config, web validation
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "WEB: SITEMAP & VALIDATION" "Sitemap config, web validation"

subheader "STEP 1: Sitemap Configuration"
run_cli_indented web sitemap show
assert_min "$(cli_json web sitemap show | jq '.items | length')" \
  1 "sitemap has at least one static route"

subheader "STEP 2: Validate Web Configuration"
run_cli_indented web validate
# `web validate` is a sections card; valid must be true. Warnings (unknown
# content-type references from shipped templates) are non-fatal and shown above.
assert_eq "$(cli_json web validate | jq -r '.sections[]|select(.heading=="valid").content')" \
  "true" "web configuration is valid"

header "WEB VALIDATION DEMO COMPLETE"
