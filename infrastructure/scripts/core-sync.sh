#!/bin/bash
# SystemPrompt Template - Core Update Script
# Usage: ./infrastructure/scripts/core-sync.sh [branch|tag]
#
# This script updates the core/ submodule and Cargo dependencies.
# The core/ directory is READ-ONLY - do not make direct modifications to it.

set -e

BRANCH="${1:-main}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=========================================="
echo "  Core Submodule Sync"
echo "=========================================="
echo ""
echo "Branch/Tag: $BRANCH"
echo ""

# Update submodule
echo "Updating core submodule..."
cd core
git fetch origin
git checkout "origin/$BRANCH" 2>/dev/null || git checkout "$BRANCH"
cd ..

echo ""
echo "Updating Cargo dependencies..."
cargo update

echo ""
echo -e "${GREEN}Core synced successfully!${NC}"
echo ""
echo "Current version:"
cd core && git describe --tags --always
cd ..
echo ""
echo "Don't forget to rebuild:"
echo "  just build"
