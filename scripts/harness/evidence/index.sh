#!/usr/bin/env bash
# index.sh: writes <evidence folder>/index.html from summary.tsv, meta.txt and screenshots/ (called by run).
#
# Usage:
#   scripts/harness/evidence/index.sh EVIDENCE_FOLDER
set -euo pipefail

out="${1:?usage: index.sh EVIDENCE_FOLDER}"
html="$out/index.html"

esc() {
    local text="$1"
    text="${text//&/&amp;}"
    text="${text//</&lt;}"
    text="${text//>/&gt;}"
    printf '%s' "$text"
}

{
    echo '<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><title>Harness evidence</title>'
    echo '<style>body{font:14px system-ui,sans-serif;margin:2em;max-width:72em}table{border-collapse:collapse}td,th{border:1px solid #bbb;padding:.3em .6em;text-align:left;vertical-align:top}.passed,.captured{color:#176f2c}.failed,.blocked{color:#b3261e;font-weight:bold}.skipped,.manual{color:#666}img{max-width:22em;border:1px solid #ccc}</style></head><body>'
    echo "<h1>Harness evidence: $(esc "$(basename "$out")")</h1>"
    echo '<h2>Machine</h2><pre>'
    esc "$(cat "$out/meta.txt")"
    echo '</pre><h2>Steps</h2><table><tr><th>Step</th><th>Status</th><th>Exit</th><th>Seconds</th><th>Log</th></tr>'
    tail -n +2 "$out/summary.tsv" | while IFS='|' read -r step status code seconds; do
        echo "<tr><td>$(esc "$step")</td><td class=\"$status\">$status</td><td>$code</td><td>$seconds</td><td><a href=\"logs/$step.log\">logs/$step.log</a></td></tr>"
    done
    echo '</table>'
    tour="$out/screenshots/tour.tsv"
    if [ -f "$tour" ]; then
        echo "<h2>Screenshot tour</h2><p>$(grep -c '|captured|' "$tour" || true) captured, $(grep -c -e '|failed|' -e '|blocked|' "$tour" || true) failed or blocked, $(grep -c '|manual|' "$tour" || true) on the manual walk. <a href=\"screenshots/tour.tsv\">tour.tsv</a>, <a href=\"screenshots/manual-walk.txt\">manual-walk.txt</a></p>"
        echo '<table><tr><th>#</th><th>State</th><th>Theme</th><th>Status</th><th>Capture or note</th></tr>'
        tail -n +2 "$tour" | while IFS='|' read -r number slug theme status file note; do
            cell="$(esc "$note")"
            [ -n "$file" ] && cell="<a href=\"screenshots/$file\"><img src=\"screenshots/$file\" alt=\"$slug $theme\"></a><br>$cell"
            echo "<tr><td>$number</td><td>$(esc "$slug")</td><td>$theme</td><td class=\"$status\">$status</td><td>$cell</td></tr>"
        done
        echo '</table>'
    fi
    if [ -f "$out/package/SHA256SUMS" ]; then
        echo '<h2>Package</h2><pre>'
        esc "$(cat "$out/package/SHA256SUMS")"
        echo '</pre>'
    fi
    echo '<p>Local evidence; nothing in this folder was uploaded.</p></body></html>'
} >"$html"
