#!/usr/bin/env bash
# Tracked prose stays where prose belongs. Ported from systemprompt-core's
# `just lint-repo-hygiene`, with this repo's sanctioned set: the product ships
# prose (services/ and storage/ skills and content, docs/, demo/, deploy/
# recipes, examples/, the pdf/ factsheets, vendored crates' READMEs, test
# fixtures, the scripts/*.txt allowlists), so those trees are allowed and
# anything else is not.
#
# Fails on:
#   * a tracked .md/.txt/.org/.rst/.adoc/.pdf/.docx outside the sanctioned set;
#   * a status/plan/report/summary/progress/findings-named prose file anywhere —
#     working notes are never committed, whichever tree they land in.
set -euo pipefail

cd "$(dirname "$0")/.."

prose='\.(md|txt|org|rst|adoc|pdf|docx)$'
sanctioned='^(docs|demo|services|storage|deploy|examples|pdf|vendor|tests)/|^scripts/[^/]+\.txt$|^(AGENTS|CLAUDE|SECURITY)\.md$|(^|/)(README|CHANGELOG)\.md$'
hits=$( { git ls-files | grep -E "$prose" | grep -vE "$sanctioned" || true
          git ls-files | grep -iE "(status|plan|report|summary|progress|findings)[^/]*${prose}" || true
        } | sort -u)
if [ -n "$hits" ]; then
    echo "lint-repo-hygiene: prose files outside the sanctioned set:" >&2
    printf '%s\n' "$hits" | sed 's/^/    /' >&2
    exit 1
fi
echo "lint-repo-hygiene: no stray status/plan/report docs tracked"
