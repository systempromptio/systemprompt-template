#!/usr/bin/env bash
# Regenerate terminal SVG recordings for every svg-*.sh script.
#
# Pipeline per scene:
#   1. asciinema rec --command ./svg-*.sh            → recordings/<name>.cast
#   2. svg-term --profile dark.xrdb                   → dark SVG
#   3. svg-term --profile light.xrdb                  → light SVG
#
# NOTE: svgo is intentionally NOT run here. svgo's preset-default strips the
# inline style="" attribute that svg-term puts the animation-* properties on,
# freezing every animation at frame 0. The files are served gzipped anyway —
# the 1–2% savings aren't worth broken animations.
#
# Two classes of scripts:
#   - Template-facing: svg-NN-*.sh at the recorder root.
#     Output: demo/recording/svg/output/{dark,light}/<name>.svg
#   - Core-facing:     crates/svg-<layer>-<name>.sh.
#     Output: ../systemprompt-core/assets/readme/terminals/{dark,light}/<layer>-<name>.svg
#             AND demo/recording/svg/output/crates/{dark,light}/<layer>-<name>.svg (reference copy)
#
# Prerequisites: asciinema, svg-term-cli, live server on :8080, demo/.token.
#
# Usage:
#   ./record.sh                         # everything (template + crates)
#   ./record.sh 03 07                   # template scenes 03 and 07 only
#   ./record.sh --crates domain-ai      # crate scenes matching these names
#   ./record.sh --dry-run               # list what would be recorded, no execution
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RECORDINGS_DIR="$SCRIPT_DIR/recordings"
OUTPUT_DARK="$SCRIPT_DIR/output/dark"
OUTPUT_LIGHT="$SCRIPT_DIR/output/light"
OUTPUT_CRATES_DARK="$SCRIPT_DIR/output/crates/dark"
OUTPUT_CRATES_LIGHT="$SCRIPT_DIR/output/crates/light"
DARK_PROFILE="$SCRIPT_DIR/profiles/dark.xrdb"
LIGHT_PROFILE="$SCRIPT_DIR/profiles/light.xrdb"
CORE_REPO="$(cd "$SCRIPT_DIR/../../.." && pwd)/../systemprompt-core"
CORE_DARK="$CORE_REPO/assets/readme/terminals/dark"
CORE_LIGHT="$CORE_REPO/assets/readme/terminals/light"

WIDTH=100
HEIGHT=32
PADDING=16

DRY_RUN=0
CRATES_FILTER=()
NUMERIC_FILTER=()
CRATES_MODE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    --crates)  CRATES_MODE=1; shift
               while [[ $# -gt 0 && "$1" != --* ]]; do CRATES_FILTER+=("$1"); shift; done ;;
    --help|-h) sed -n '2,30p' "$0"; exit 0 ;;
    *)         NUMERIC_FILTER+=("$1"); shift ;;
  esac
done

need() { command -v "$1" >/dev/null 2>&1 || { echo "ERROR: $1 not found. $2" >&2; exit 1; }; }

if [[ $DRY_RUN -eq 0 ]]; then
  need asciinema "Install with: pipx install asciinema"
  need svg-term  "Install with: npm i -g svg-term-cli"

  if ! curl -sf -o /dev/null http://localhost:8080/; then
    echo "ERROR: server not responding at http://localhost:8080 — run: just start" >&2
    exit 1
  fi
  if [[ ! -f "$SCRIPT_DIR/../../.token" ]]; then
    echo "ERROR: demo/.token missing — run: ./demo/00-preflight.sh" >&2
    exit 1
  fi
  if [[ ! -d "$CORE_REPO" ]]; then
    echo "ERROR: sibling repo $CORE_REPO not found — clone systemprompt-core alongside this repo" >&2
    exit 1
  fi

  mkdir -p "$RECORDINGS_DIR" "$OUTPUT_DARK" "$OUTPUT_LIGHT" \
           "$OUTPUT_CRATES_DARK" "$OUTPUT_CRATES_LIGHT" \
           "$CORE_DARK" "$CORE_LIGHT"
fi

shopt -s nullglob
TEMPLATE_SCRIPTS=("$SCRIPT_DIR"/svg-[0-9][0-9]-*.sh)
CRATE_SCRIPTS=("$SCRIPT_DIR"/crates/svg-*.sh)
shopt -u nullglob

# Apply template numeric filter
SELECTED_TEMPLATE=()
if [[ ${#NUMERIC_FILTER[@]} -gt 0 ]]; then
  FILTER=""
  for n in "${NUMERIC_FILTER[@]}"; do FILTER="$FILTER|svg-$(printf '%02d' "$n")-"; done
  FILTER="${FILTER#|}"
  for s in "${TEMPLATE_SCRIPTS[@]}"; do
    [[ "$(basename "$s")" =~ $FILTER ]] && SELECTED_TEMPLATE+=("$s")
  done
elif [[ $CRATES_MODE -eq 0 ]]; then
  SELECTED_TEMPLATE=("${TEMPLATE_SCRIPTS[@]}")
fi

# Apply crates filter
SELECTED_CRATES=()
if [[ $CRATES_MODE -eq 1 ]]; then
  if [[ ${#CRATES_FILTER[@]} -eq 0 ]]; then
    SELECTED_CRATES=("${CRATE_SCRIPTS[@]}")
  else
    for s in "${CRATE_SCRIPTS[@]}"; do
      base="$(basename "$s" .sh)"; base="${base#svg-}"
      for pat in "${CRATES_FILTER[@]}"; do
        [[ "$base" == *"$pat"* ]] && SELECTED_CRATES+=("$s") && break
      done
    done
  fi
elif [[ ${#NUMERIC_FILTER[@]} -eq 0 ]]; then
  # No explicit filter → record crates too
  SELECTED_CRATES=("${CRATE_SCRIPTS[@]}")
fi

TOTAL=$(( ${#SELECTED_TEMPLATE[@]} + ${#SELECTED_CRATES[@]} ))
if [[ $TOTAL -eq 0 ]]; then
  echo "ERROR: no scripts matched filters" >&2
  exit 1
fi

capture() {
  local script="$1" cast="$2"
  asciinema rec \
    --overwrite \
    --cols "$WIDTH" \
    --rows "$HEIGHT" \
    --command "bash '$script'" \
    --env "TERM,SHELL" \
    --idle-time-limit 2 \
    --quiet \
    "$cast"
}

render() {
  local cast="$1" out="$2" profile="$3"
  svg-term \
    --in "$cast" \
    --out "$out" \
    --profile "$profile" \
    --term xresources \
    --window \
    --width "$WIDTH" \
    --height "$HEIGHT" \
    --padding "$PADDING"
}

record_template() {
  local script="$1"
  local base cast dark_svg light_svg
  base="$(basename "$script" .sh)"; base="${base#svg-}"
  cast="$RECORDINGS_DIR/$base.cast"
  dark_svg="$OUTPUT_DARK/$base.svg"
  light_svg="$OUTPUT_LIGHT/$base.svg"
  echo ""; echo "━━━ template: $base ━━━"
  if [[ $DRY_RUN -eq 1 ]]; then
    echo "  would record → $dark_svg"
    echo "  would record → $light_svg"
    return
  fi
  capture "$script" "$cast"
  render  "$cast" "$dark_svg"  "$DARK_PROFILE"
  render  "$cast" "$light_svg" "$LIGHT_PROFILE"
}

record_crate() {
  local script="$1"
  local base cast dark_svg light_svg core_dark core_light
  base="$(basename "$script" .sh)"; base="${base#svg-}"
  cast="$RECORDINGS_DIR/crate-$base.cast"
  dark_svg="$OUTPUT_CRATES_DARK/$base.svg"
  light_svg="$OUTPUT_CRATES_LIGHT/$base.svg"
  core_dark="$CORE_DARK/$base.svg"
  core_light="$CORE_LIGHT/$base.svg"
  echo ""; echo "━━━ crate: $base ━━━"
  if [[ $DRY_RUN -eq 1 ]]; then
    echo "  would record → $core_dark"
    echo "  would record → $core_light"
    echo "  would copy   → $dark_svg / $light_svg"
    return
  fi
  capture "$script" "$cast"
  render  "$cast" "$dark_svg"  "$DARK_PROFILE"
  render  "$cast" "$light_svg" "$LIGHT_PROFILE"
  cp "$dark_svg"  "$core_dark"
  cp "$light_svg" "$core_light"
}

for s in "${SELECTED_TEMPLATE[@]}"; do record_template "$s"; done
for s in "${SELECTED_CRATES[@]}";   do record_crate    "$s"; done

echo ""
if [[ $DRY_RUN -eq 1 ]]; then
  echo "Dry run. ${#SELECTED_TEMPLATE[@]} template + ${#SELECTED_CRATES[@]} crate scenes would be recorded."
else
  echo "Done. ${#SELECTED_TEMPLATE[@]} template + ${#SELECTED_CRATES[@]} crate recordings generated."
fi
