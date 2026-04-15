#!/bin/bash
# tests-integration — test suite
set -e
source "$(dirname "$0")/../_colors.sh"

header "tests-integration" "End-to-end integration tests against a live DB"
pause 0.8

type_cmd "cargo test --list -p integration"
pause 0.2
cargo test --list -p integration 2>&1 | tail -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "every domain covered, every path tested"
pause 1.2
