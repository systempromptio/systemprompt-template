#!/usr/bin/env bash
# Switches the working tree between the committed manifests (core override set
# inert) and a local build against the sibling core checkout they name. The
# root workspace and the separate tests/ workspace carry the same set, so both
# switch together.
#   scripts/core-patch.sh on|off|status
set -euo pipefail
cd "$(dirname "$0")/.."
inert='[workspace.metadata.unreleased-core-patch]'
live='[patch.crates-io]'
manifests=(Cargo.toml tests/Cargo.toml)
locks=(Cargo.toml Cargo.lock tests/Cargo.toml tests/Cargo.lock)
case "${1:-status}" in
  on)
    grep -qxF "$inert" Cargo.toml || { echo "core patch is already live"; exit 0; }
    for m in "${manifests[@]}"; do
      sed -i "s|^\[workspace\.metadata\.unreleased-core-patch\]$|$live|" "$m"
    done
    git update-index --skip-worktree "${locks[@]}"
    echo "core patch live; the root and tests/ manifests and locks are hidden from git"
    ;;
  off)
    git update-index --no-skip-worktree "${locks[@]}"
    for m in "${manifests[@]}"; do
      sed -i "s|^\[patch\.crates-io\]$|$inert|" "$m"
    done
    echo "core patch inert; the root and tests/ manifests and locks are visible to git"
    ;;
  status)
    if grep -qxF "$live" Cargo.toml; then echo live; else echo inert; fi
    ;;
  *) echo "usage: scripts/core-patch.sh on|off|status" >&2; exit 2 ;;
esac
