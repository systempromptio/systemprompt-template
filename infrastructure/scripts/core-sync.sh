#!/bin/bash
# SystemPrompt Template - Core Subtree Sync Script
# Usage: ./infrastructure/scripts/core-sync.sh [branch|tag]
#
# This script updates the core/ subtree from the upstream systemprompt-core repository.
# The core/ directory is READ-ONLY - do not make direct modifications to it.

set -e

CORE_REMOTE="https://github.com/systempromptio/systemprompt-core.git"
BRANCH="${1:-main}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=========================================="
echo "  Core Subtree Sync"
echo "=========================================="
echo ""
echo "Remote: $CORE_REMOTE"
echo "Branch: $BRANCH"
echo ""

# Check for uncommitted changes in core/
if git diff --name-only HEAD 2>/dev/null | grep -q "^core/"; then
    echo -e "${RED}ERROR: Uncommitted changes in core/ directory${NC}"
    echo ""
    echo "The core/ directory is READ-ONLY. You should not modify files there."
    echo ""
    echo "To discard your changes:"
    echo "  git checkout -- core/"
    echo ""
    exit 1
fi

# Check for staged changes in core/
if git diff --cached --name-only 2>/dev/null | grep -q "^core/"; then
    echo -e "${RED}ERROR: Staged changes in core/ directory${NC}"
    echo ""
    echo "The core/ directory is READ-ONLY. You should not modify files there."
    echo ""
    echo "To unstage your changes:"
    echo "  git reset HEAD -- core/"
    echo ""
    exit 1
fi

echo "Pulling latest from $BRANCH..."
echo ""

# Pull subtree updates
git subtree pull --prefix=core "$CORE_REMOTE" "$BRANCH" --squash \
    -m "chore: sync core from systemprompt-core ($BRANCH)"

echo ""
echo -e "${GREEN}Core synced successfully!${NC}"
echo ""
echo "Don't forget to rebuild:"
echo "  just build"
