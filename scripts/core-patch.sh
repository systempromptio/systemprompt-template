#!/usr/bin/env bash
# Switches the working tree between the committed manifest (core override set
# inert) and a local build against the sibling core checkout it names.
#   scripts/core-patch.sh on|off|status
set -euo pipefail
cd "$(dirname "$0")/.."
inert='[workspace.metadata.unreleased-core-patch]'
live='[patch.crates-io]'
case "${1:-status}" in
  on)
    grep -qxF "$inert" Cargo.toml || { echo "core patch is already live"; exit 0; }
    sed -i "s|^\[workspace\.metadata\.unreleased-core-patch\]$|$live|" Cargo.toml
    git update-index --skip-worktree Cargo.toml Cargo.lock
    echo "core patch live; Cargo.toml and Cargo.lock are hidden from git"
    ;;
  off)
    git update-index --no-skip-worktree Cargo.toml Cargo.lock
    sed -i "s|^\[patch\.crates-io\]$|$inert|" Cargo.toml
    echo "core patch inert; Cargo.toml and Cargo.lock are visible to git"
    ;;
  status)
    if grep -qxF "$live" Cargo.toml; then echo live; else echo inert; fi
    ;;
  *) echo "usage: scripts/core-patch.sh on|off|status" >&2; exit 2 ;;
esac
