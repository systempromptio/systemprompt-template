#!/bin/bash
# SVG RECORDING: Web configuration surface
# content-types, sitemap, validation — the CMS-facing surface of the web extension.
set -e
source "$(dirname "$0")/_colors.sh"

header "WEB EXTENSION" "content-types, sitemap, template validation"
pause 1

subheader "Content types" "declared in services/web/config.yaml — discovered at load"
pause 0.3
type_cmd "systemprompt web content-types list"
pause 0.3
"$CLI" web content-types list --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" | head -12 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 1

divider

subheader "Sitemap" "every live URL, generated from content + routes"
pause 0.3
type_cmd "systemprompt web sitemap show"
pause 0.3
"$CLI" web sitemap show --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" | head -12 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 1

divider

subheader "Validate" "templates, assets, references — fail loud"
pause 0.3
type_cmd "systemprompt web validate"
pause 0.3
"$CLI" web validate --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" | head -10 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "content-types, sitemap, and templates all verified at load time"
pause 1.5
