#!/bin/bash
# SVG RECORDING: Users + IP bans
# Governance follows identity. Users, roles, and bans are first-class.
set -e
source "$(dirname "$0")/_colors.sh"

header "USERS & IDENTITY" "Governance follows the user — not the client"
pause 1

# ── User list ──
subheader "Users" "first-class identities, not session cookies"
pause 0.3

type_cmd "systemprompt admin users list"
pause 0.3

"$CLI" admin users list --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -10 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
pause 1

divider

# ── Roles ──
subheader "Roles" "scopes enforce what tools a user's agents may call"
pause 0.3

type_cmd "systemprompt admin users stats"
pause 0.3

"$CLI" admin users stats --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -10 \
  | while IFS= read -r line; do printf "    ${CYAN}%s${R}\n" "$line"; done
echo ""
pass "admin / developer / associate — scope-checked on every tool call"
pause 1

divider

# ── IP ban ──
subheader "IP bans" "network-layer enforcement, audited"
pause 0.3

type_cmd "systemprompt admin users ban add 192.168.99.99 --reason \"svg demo\""
pause 0.3

"$CLI" admin users ban add 192.168.99.99 --reason "svg demo" --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -6 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
fail "192.168.99.99 — banned, all requests from this IP rejected"
pause 0.8

type_cmd "systemprompt admin users ban remove 192.168.99.99"
pause 0.3

"$CLI" admin users ban remove 192.168.99.99 --yes --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -4 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
pass "every ban + unban recorded to the audit trail"
pause 1.5
