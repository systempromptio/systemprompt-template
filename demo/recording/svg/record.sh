#!/usr/bin/env bash
# Regenerate terminal SVG recordings for every svg-NN-*.sh script.
#
# Pipeline:
#   1. asciinema rec --command ./svg-NN-*.sh  →  recordings/NN-*.cast
#   2. svg-term --in .cast --profile dark.xrdb   →  output/dark/NN-*.svg
#   3. svg-term --in .cast --profile light.xrdb  →  output/light/NN-*.svg
#
# Prerequisites: asciinema, svg-term-cli (npm i -g svg-term-cli), live server
# on localhost:8080, and a token in demo/.token (run ./demo/00-preflight.sh).
#
# Usage:  ./record.sh              # regenerate all
#         ./record.sh 03 07        # regenerate only the given numbers
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RECORDINGS_DIR="$SCRIPT_DIR/recordings"
OUTPUT_DARK="$SCRIPT_DIR/output/dark"
OUTPUT_LIGHT="$SCRIPT_DIR/output/light"
DARK_PROFILE="$SCRIPT_DIR/profiles/dark.xrdb"
LIGHT_PROFILE="$SCRIPT_DIR/profiles/light.xrdb"

WIDTH=100
HEIGHT=32
PADDING=16

mkdir -p "$RECORDINGS_DIR" "$OUTPUT_DARK" "$OUTPUT_LIGHT"

need() { command -v "$1" >/dev/null 2>&1 || { echo "ERROR: $1 not found. $2" >&2; exit 1; }; }
need asciinema "Install with: pipx install asciinema"
need svg-term   "Install with: npm i -g svg-term-cli"

if ! curl -sf -o /dev/null http://localhost:8080/; then
  echo "ERROR: server not responding at http://localhost:8080 — run: just start" >&2
  exit 1
fi

if [[ ! -f "$SCRIPT_DIR/../../.token" ]]; then
  echo "ERROR: demo/.token missing — run: ./demo/00-preflight.sh" >&2
  exit 1
fi

shopt -s nullglob
ALL_SCRIPTS=("$SCRIPT_DIR"/svg-[0-9][0-9]-*.sh)
shopt -u nullglob

if [[ ${#ALL_SCRIPTS[@]} -eq 0 ]]; then
  echo "ERROR: no svg-NN-*.sh scripts found in $SCRIPT_DIR" >&2
  exit 1
fi

# Filter by numeric prefix if arguments were passed.
if [[ $# -gt 0 ]]; then
  FILTER=""
  for n in "$@"; do FILTER="$FILTER|svg-$(printf '%02d' "$n")-"; done
  FILTER="${FILTER#|}"
  SCRIPTS=()
  for s in "${ALL_SCRIPTS[@]}"; do
    [[ "$(basename "$s")" =~ $FILTER ]] && SCRIPTS+=("$s")
  done
else
  SCRIPTS=("${ALL_SCRIPTS[@]}")
fi

record_one() {
  local script="$1"
  local base cast dark_svg light_svg
  base="$(basename "$script" .sh)"
  base="${base#svg-}"                 # 01-governance
  cast="$RECORDINGS_DIR/$base.cast"
  dark_svg="$OUTPUT_DARK/$base.svg"
  light_svg="$OUTPUT_LIGHT/$base.svg"

  echo ""
  echo "━━━ $base ━━━"

  asciinema rec \
    --overwrite \
    --cols "$WIDTH" \
    --rows "$HEIGHT" \
    --command "bash '$script'" \
    --env "TERM,SHELL" \
    --idle-time-limit 2 \
    --quiet \
    "$cast"

  svg-term \
    --in "$cast" \
    --out "$dark_svg" \
    --profile "$DARK_PROFILE" \
    --term xresources \
    --window \
    --width "$WIDTH" \
    --height "$HEIGHT" \
    --padding "$PADDING"

  svg-term \
    --in "$cast" \
    --out "$light_svg" \
    --profile "$LIGHT_PROFILE" \
    --term xresources \
    --window \
    --width "$WIDTH" \
    --height "$HEIGHT" \
    --padding "$PADDING"

  echo "  ✓ $cast"
  echo "  ✓ $dark_svg"
  echo "  ✓ $light_svg"
}

for s in "${SCRIPTS[@]}"; do record_one "$s"; done

echo ""
echo "Done. ${#SCRIPTS[@]} recording(s) regenerated."
