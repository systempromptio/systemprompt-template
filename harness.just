# Recipes for the desktop shell workspace (extensions/harness), a cargo
# workspace separate from the root one with its own lockfile. Imported by the
# justfile; every recipe is named harness-*. The workspace has no members yet,
# so there is nothing to compile: these read the toolchain and the resolution.

HARNESS_MANIFEST := "extensions/harness/Cargo.toml"

# Print the toolchain the desktop shell workspace resolves (rustc, cargo)
harness-toolchain:
    bash scripts/harness/toolchain.sh

# Read the desktop shell workspace manifest against its lockfile (no compile).
# Cargo refuses to resolve a workspace with no members, so this is --no-deps
# until the first shell crate lands.
harness-resolve:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo metadata --manifest-path "{{HARNESS_MANIFEST}}" --locked --no-deps --format-version 1 > /dev/null
    echo "harness: {{HARNESS_MANIFEST}} resolves against its Cargo.lock"
