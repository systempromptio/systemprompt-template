#!/usr/bin/env bash
# Write the instance's per-release run record into a kit release's notes.
#
# Usage: tools/kit-stats.sh <release-tag>
#
# Reads two tables from the instance's export API with a personal access token
# (the only admin surface a PAT is accepted on): the release-impact table, one
# row per kit version, and the same split per skill. Both carry no person,
# prompt or transcript. The markdown lands between two markers in the release
# notes, so a re-run replaces the section instead of appending a second one;
# the JSON is attached to the release as `kit-stats.json`.
#
# Env: API_URL (instance base URL), STATS_PAT (a PAT whose owner has console
# access), GH_TOKEN. Without API_URL or STATS_PAT it warns and exits 0 — stats
# never block a release.
set -euo pipefail

tag="${1:?usage: kit-stats.sh <release-tag>}"
if [ -z "${API_URL:-}" ] || [ -z "${STATS_PAT:-}" ]; then
  echo "::warning title=No kit stats::set SYSTEMPROMPT_API_URL and KIT_STATS_PAT to record each release's run figures in its notes"
  exit 0
fi

marketplace=$(jq -r '.name' .claude-plugin/marketplace.json)
base="${API_URL%/}/admin/export/analysis-kit-release-impact?marketplace=${marketplace}&days=365"
work=$(mktemp -d)

fetch() {
  curl -fsS --retry 2 -H "Authorization: Bearer $STATS_PAT" "$1" -o "$2" || {
    echo "::warning title=Kit stats unavailable::the instance refused or failed $1 — is the PAT's owner a console user?"
    exit 0
  }
}

fetch "${base}&per_skill=false&format=markdown" "$work/versions.md"
fetch "${base}&per_skill=false&format=json" "$work/versions.json"
fetch "${base}&per_skill=true&format=json" "$work/skills.json"

jq -n --arg tag "$tag" --arg marketplace "$marketplace" \
  --arg at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --slurpfile versions "$work/versions.json" --slurpfile skills "$work/skills.json" \
  '{tag: $tag, marketplace: $marketplace, recorded_at: $at, window_days: 365,
    versions: $versions[0], per_skill: $skills[0]}' > "$work/kit-stats.json"

{
  echo "<!-- kit-stats:start -->"
  echo "## Run record by kit version"
  echo
  echo "Every version of \`$marketplace\` with runs on the instance in the last 365 days, as of $(date -u +%Y-%m-%d). A version released today has no runs yet; the rows before it are what it is measured against. Per-skill figures: \`kit-stats.json\`."
  echo
  if [ "$(jq length "$work/versions.json")" -eq 0 ]; then
    echo "_No runs recorded yet._"
  else
    cat "$work/versions.md"
  fi
  echo "<!-- kit-stats:end -->"
} > "$work/section.md"

gh release view "$tag" --json body -q .body > "$work/body.md"
python3 - "$work/body.md" "$work/section.md" <<'PY'
import re, sys
body, section = (open(p).read() for p in sys.argv[1:3])
pattern = re.compile(r"<!-- kit-stats:start -->.*?<!-- kit-stats:end -->\n?", re.S)
body = pattern.sub(lambda _: section, body) if pattern.search(body) else body.rstrip() + "\n\n" + section
open(sys.argv[1], "w").write(body)
PY
gh release edit "$tag" --notes-file "$work/body.md"
gh release upload "$tag" "$work/kit-stats.json" --clobber

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  sed '/^<!-- kit-stats/d' "$work/section.md" >> "$GITHUB_STEP_SUMMARY"
fi
echo "::notice title=Kit stats::recorded $(jq length "$work/versions.json") version row(s) on $tag"
