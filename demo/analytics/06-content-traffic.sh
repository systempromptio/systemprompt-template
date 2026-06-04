#!/bin/bash
# DEMO: CONTENT & TRAFFIC ANALYTICS
# Content engagement, top pages, traffic sources, geographic distribution, devices
#
# What this does:
#   1. Shows content stats
#   2. Lists top content (top 20 lines)
#   3. Shows content trends over 7 days
#   4. Shows traffic sources
#   5. Shows geographic distribution
#   6. Shows device breakdown
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

# Show a traffic rollup (box table) and assert it has at least one row in the
# structured --json output. 01-seed-data.sh inserts 100 synthetic sessions, so
# every rollup must be non-empty; an empty one means the seed never landed.
traffic_rollup() {
  local sub="$1" label="$2"
  run_cli_indented analytics traffic "$sub"
  assert_min "$(cli_json analytics traffic "$sub" | jq '.items | length')" 1 "$label"
}

header "ANALYTICS: CONTENT & TRAFFIC" "Content engagement, top pages, traffic sources, geo, devices"

subheader "STEP 1: Content Stats"
run_cli_indented analytics content stats

subheader "STEP 2: Top Content"
info "Top content ranks markdown_content rows by tracked page_view events."
info "Synthetic seed traffic targets dashboard/admin paths, not blog slugs — this demo surfaces the query shape rather than a populated ranking."
"$CLI" analytics content top --profile "$PROFILE" 2>&1 \
  | grep -v -E 'No content found|No matching' \
  | sed 's/^/  /'
echo ""

subheader "STEP 3: Content Trends"
run_cli_indented analytics content trends --since 7d

subheader "STEP 4: Traffic Sources"
traffic_rollup sources "traffic sources rollup has rows (seeded sessions)"

subheader "STEP 5: Geographic Distribution"
traffic_rollup geo "geographic rollup has rows (seeded sessions)"

subheader "STEP 6: Device Breakdown"
traffic_rollup devices "device breakdown has rows (seeded sessions)"

header "CONTENT & TRAFFIC DEMO COMPLETE"
