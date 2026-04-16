#!/bin/bash
# SVG RECORDING: Deploy Anywhere
# Same binary, same CLI — local, Docker, cloud, air-gapped.
set -e
source "$(dirname "$0")/_colors.sh"

header "DEPLOY ANYWHERE" "Same binary, same CLI, any environment"
pause 1

# ── Profiles ──
subheader "Profiles" "one binary switches between environments"
pause 0.3

type_cmd "systemprompt admin session list"
pause 0.3
"$CLI" admin session list --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -15 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 1

divider

# ── Config ──
subheader "Configuration" "environment-agnostic — config follows the profile"
pause 0.3

type_cmd "systemprompt admin config show"
pause 0.3
"$CLI" admin config show --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -12 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 1

divider

# ── Paths ──
subheader "Paths" "every path relative — portable across machines"
pause 0.3

type_cmd "systemprompt admin config paths show"
pause 0.3
"$CLI" admin config paths show --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -10 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
check "same binary runs local, Docker, cloud, or air-gapped"
pause 1.5
