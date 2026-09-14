#!/usr/bin/env bash
# Capture the required desktop/mobile operator evidence with an authenticated
# Playwright storage state produced by the acceptance login step.
set -euo pipefail

: "${BASE_URL:?BASE_URL is required}"
: "${AUTH_STATE:?AUTH_STATE is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
: "${EXPERIMENT_ID:?EXPERIMENT_ID is required}"
mkdir -p "$EVIDENCE_DIR/screens"

pages=(
    "evaluations|/admin/analysis/evaluations"
    "comparison|/admin/analysis/evaluations/$EXPERIMENT_ID"
    "publication|/admin/analysis/publications"
    "installation|/admin/analysis/publications#installation-receipts"
    "rollback|/admin/analysis/publications#publication-history"
    "version-impact|/admin/analysis/impact"
)
for item in "${pages[@]}"; do
    name="${item%%|*}"
    path="${item#*|}"
    playwright screenshot --browser chromium --full-page --load-storage "$AUTH_STATE" \
        --viewport-size '1440,1000' "$BASE_URL$path" "$EVIDENCE_DIR/screens/$name-desktop.png"
    playwright screenshot --browser chromium --full-page --load-storage "$AUTH_STATE" \
        --device 'Pixel 7' "$BASE_URL$path" "$EVIDENCE_DIR/screens/$name-mobile.png"
done
sha256sum "$EVIDENCE_DIR"/screens/*.png > "$EVIDENCE_DIR/screens/SHA256SUMS"
