#!/bin/bash
# app-generator — static site generation
set -e
source "$(dirname "$0")/../_colors.sh"

header "app-generator" "Prerenders templates into a static site"
pause 0.8

type_cmd "systemprompt web templates list"
pause 0.2
"$CLI" web templates list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt web sitemap show"
pause 0.2
"$CLI" web sitemap show --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "SSR at runtime, prerender for the edge"
pause 1.2
