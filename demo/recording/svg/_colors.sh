#!/bin/bash
# Shared color constants and utility functions for SVG demo recordings

# ── Colors (minimal set that renders cleanly in svg-term) ──
GREEN='\033[32m'
RED='\033[31m'
CYAN='\033[36m'
YELLOW='\033[33m'
WHITE='\033[97m'
BOLD='\033[1m'
DIM='\033[2m'
R='\033[0m'

# ── Project setup ──
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RECORDING_DIR="$(dirname "$SCRIPT_DIR")"
DEMO_DIR="$(dirname "$RECORDING_DIR")"
PROJECT_DIR="$(dirname "$DEMO_DIR")"
CLI="$PROJECT_DIR/target/debug/systemprompt"
if [[ -x "$PROJECT_DIR/target/release/systemprompt" && "$PROJECT_DIR/target/release/systemprompt" -nt "$CLI" ]]; then
  CLI="$PROJECT_DIR/target/release/systemprompt"
fi
export RUST_LOG=warn

PROFILE="${1:-local}"
BASE_URL="http://localhost:8080"

TOKEN_FILE="$DEMO_DIR/.token"
if [[ -f "$TOKEN_FILE" ]]; then
  TOKEN=$(cat "$TOKEN_FILE")
fi

# ── Utility functions ──

header() {
  echo ""
  printf "${BOLD}${WHITE}%s${R}\n" "$1"
  printf "${DIM}%s${R}\n" "$2"
  printf "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${R}\n"
  echo ""
}

subheader() {
  printf "${CYAN}──${R} ${BOLD}${WHITE}%s${R}\n" "$1"
  if [[ -n "${2:-}" ]]; then
    printf "${DIM}%s${R}\n" "   $2"
  fi
  echo ""
}

divider() {
  echo ""
  printf "${DIM}────────────────────────────────────────────────────────────${R}\n"
  echo ""
}

pass() {
  printf "${GREEN}${BOLD}>>> ALLOW${R} ${GREEN}%s${R}\n" "$1"
}

fail() {
  printf "${RED}${BOLD}>>> DENY${R}  ${RED}%s${R}\n" "$1"
}

check() {
  printf "${GREEN}${BOLD}>>>   OK${R}  ${GREEN}%s${R}\n" "$1"
}

info() {
  printf "${DIM}%s${R}\n" "$1"
}

type_cmd() {
  local cmd="$1"
  printf "${GREEN}\$${R} "
  local i=0
  while [ $i -lt ${#cmd} ]; do
    printf "%s" "${cmd:$i:1}"
    sleep 0.015
    i=$((i+1))
  done
  printf "\n"
  sleep 0.2
}

color_json() {
  python3 -c "
import sys, json, re
data = sys.stdin.read()
try:
    obj = json.loads(data)
    pretty = json.dumps(obj, indent=2)
except:
    pretty = data
C='\033[36m'
G='\033[32m'
E='\033[31m'
N='\033[0m'
B='\033[1m'
for line in pretty.split('\n'):
    if '\"allow\"' in line:
        line = line.replace('\"allow\"', f'{B}{G}\"allow\"{N}')
    elif '\"deny\"' in line:
        line = line.replace('\"deny\"', f'{B}{E}\"deny\"{N}')
    line = re.sub(r'\"(\w+)\":', f'{C}\"\g<1>\":{N}', line)
    print(f'  {line}')
" 2>/dev/null || cat
}

pause() {
  sleep "${1:-0.8}"
}

table_row() {
  printf "${DIM}%-18s${R} ${3:-}%s${R}\n" "$1" "$2"
}

table_top() { :; }
table_mid() { echo ""; }
table_bot() { echo ""; }
