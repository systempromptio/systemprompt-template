#!/usr/bin/env bash
# Pre-merge gate (ported from core's lint-repo-construction.sh): repositories
# are constructed at composition roots and injected; ad-hoc
# `SomeRepository::new(&pool)` inside handler/method bodies is the
# anti-pattern this gate blocks.
#
# A `*Repository::new(` call is legal only when the file falls in one of two
# tiers:
#
#   1. Pattern-exempt (structural):
#      - **/src/repository/**            bundles and sub-repo construction
#      - extensions/web/src/router/**    the router-state composition root
#      - **/jobs/**                      per-tick scheduler job bodies
#      - extensions/mcp/*/src/server/**  MCP server composition roots
#      - extensions/cli/**               one-shot command bodies
#
#   2. The explicit file list below. Every entry carries its reason. Adding a
#      file requires justification in review — the default for new code is to
#      take the repository from the owning service's stored field, constructed
#      once where the service is built.
#
# Bundle constructors (`*Repositories::new(`) never match the regex — the
# plural suffix is the sanctioned composition form.
#
# The allowlist is checked for staleness: an entry whose file is gone or no
# longer constructs a repository fails the gate.
#
# Test workspaces are out of scope: fixtures construct repositories freely.
set -euo pipefail

cd "$(dirname "$0")/.."

ALLOWED_FILES=(
  # content services construct-and-store their own repository once at service
  # build; the content crate has no shared context object to inject from.
  extensions/web/content/src/services/content.rs
  extensions/web/content/src/services/ingestion.rs
  extensions/web/content/src/services/link.rs
  extensions/web/content/src/services/link_analytics.rs
  extensions/web/content/src/services/link_generation.rs
  extensions/web/content/src/services/search.rs
  # pre-gate ad-hoc handler construction; shrink by injecting the repository
  # through the router state, then delete the entry.
  extensions/web/content/src/api/handlers/links.rs
)

declare -A allowed
for f in "${ALLOWED_FILES[@]}"; do
  allowed["$f"]=1
done

pattern='\b[A-Z][A-Za-z0-9_]*Repository::new('

fail=0
declare -A seen
while IFS= read -r file; do
  case "$file" in
    */src/repository/*) continue ;;
    extensions/web/src/router/*) continue ;;
    */jobs/*) continue ;;
    extensions/mcp/*/src/server/*) continue ;;
    extensions/cli/*) continue ;;
  esac
  if ! grep -q "$pattern" "$file"; then
    continue
  fi
  if [[ -n "${allowed[$file]:-}" ]]; then
    seen["$file"]=1
  else
    echo "lint-repo-construction: ad-hoc repository construction in $file"
    grep -n "$pattern" "$file" | sed 's/^/  /'
    fail=1
  fi
done < <(git ls-files -co --exclude-standard 'extensions/**/*.rs' 'src/**/*.rs' 'bridge/src/**/*.rs')

for f in "${ALLOWED_FILES[@]}"; do
  if [[ ! -f "$f" ]]; then
    echo "lint-repo-construction: stale allowlist entry (file missing): $f"
    fail=1
  elif [[ -z "${seen[$f]:-}" ]]; then
    echo "lint-repo-construction: stale allowlist entry (no repository construction): $f"
    fail=1
  fi
done

if [[ "$fail" -ne 0 ]]; then
  echo
  echo "Repositories are constructed once at a composition root (router state,"
  echo "job body, or an owning service's stored field) and injected. See"
  echo "scripts/lint-repo-construction.sh for the exemption tiers."
  exit 1
fi

echo "lint-repo-construction: OK"
