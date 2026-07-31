#!/bin/bash
# Add per-model gateway routes for the Pi demo to the active profile.
#
# The stock profile routes whole model families (claude-*, gpt-*, gemini-*),
# which makes them single governance entities. Per-model routes ahead of the
# globs (first match wins) turn each Pi demo model into its own authz entity,
# so the dashboard can enable/disable individual models per user.
#
# Idempotent: skips routes whose ids already exist. Routes materialize as
# authz entities at startup, so restart the server after running this.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$HERE/../.." && pwd)"
PROFILE="${PROFILE:-local}"
PROFILE_YAML="$PROJECT_DIR/.systemprompt/profiles/$PROFILE/profile.yaml"
[[ -f "$PROFILE_YAML" ]] || { echo "ERROR: profile not found: $PROFILE_YAML" >&2; exit 1; }

# id / model_pattern / provider triplets, one per demo model
ROUTES="pi-claude-sonnet-4-6 claude-sonnet-4-6 anthropic
pi-claude-opus-4-8 claude-opus-4-8 anthropic
pi-gpt-5-mini gpt-5-mini openai
pi-gemini-2-5-flash gemini-2.5-flash gemini
pi-gpt-oss-120b gpt-oss-120b cerebras"

BLOCK_FILE="$(mktemp)"
trap 'rm -f "$BLOCK_FILE"' EXIT
ADDED=false
while read -r id pattern provider; do
  # Why: a route naming a provider the profile does not declare fails
  # Profile::validate, so every later CLI call and the next server start abort
  # on an unreadable profile. setup-local declares only the providers it was
  # given keys for, so the demo's optional models have to be skipped rather
  # than written blind.
  if ! grep -q "^- name: $provider\$" "$PROFILE_YAML"; then
    echo "  ! skipping route $id — provider '$provider' is not declared in this profile"
    continue
  fi
  if grep -q "id: $id\$" "$PROFILE_YAML"; then
    echo " ok route $id already present"
  else
    printf '  - id: %s\n    model_pattern: %s\n    provider: %s\n' "$id" "$pattern" "$provider" >> "$BLOCK_FILE"
    ADDED=true
    echo "==> adding route $id ($pattern -> $provider)"
  fi
done <<< "$ROUTES"

if $ADDED; then
  # Insert directly under the "  routes:" key inside the gateway block so the
  # per-model routes precede the glob catch-alls (first match wins).
  TMP="$PROFILE_YAML.tmp.$$"
  awk -v blockfile="$BLOCK_FILE" '
    { print }
    !done && /^  routes:$/ {
      while ((getline line < blockfile) > 0) print line
      close(blockfile); done=1
    }
  ' "$PROFILE_YAML" > "$TMP"
  mv "$TMP" "$PROFILE_YAML"
  echo "==> profile updated: $PROFILE_YAML"
  echo "    Restart the server to materialize the routes: just start"
else
  echo " ok nothing to do"
fi
