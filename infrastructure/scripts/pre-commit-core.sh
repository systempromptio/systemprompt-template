#!/bin/bash
# SystemPrompt Template - Pre-commit Hook
# Prevents direct modifications to the core/ directory
#
# The core/ directory is a READ-ONLY git subtree from systemprompt-core.
# To update core, use: just core-sync

# Check if any staged files are in core/
CHANGED_CORE=$(git diff --cached --name-only | grep "^core/" || true)

if [ -n "$CHANGED_CORE" ]; then
    echo ""
    echo "=========================================="
    echo "  ERROR: Direct modifications to core/"
    echo "=========================================="
    echo ""
    echo "The core/ directory is READ-ONLY."
    echo "It is a git subtree from systemprompt-core."
    echo ""
    echo "The following files were modified:"
    echo "$CHANGED_CORE" | head -20

    COUNT=$(echo "$CHANGED_CORE" | wc -l)
    if [ "$COUNT" -gt 20 ]; then
        echo "... and $((COUNT - 20)) more files"
    fi

    echo ""
    echo "To update core, use:"
    echo "  just core-sync"
    echo ""
    echo "To discard your changes:"
    echo "  git checkout -- core/"
    echo "  git reset HEAD -- core/"
    echo ""
    exit 1
fi

exit 0
