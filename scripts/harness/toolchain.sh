#!/usr/bin/env bash
# Print the toolchain the desktop shell workspace builds with: rustc and cargo
# as rustup resolves them from inside extensions/harness. The workspace has no
# rust-toolchain.toml of its own, so this is the check that the template's is
# found from there. Run by `just harness-toolchain` and by every job of the
# Harness workflow (Windows, macOS, Linux).
set -euo pipefail
cd "$(dirname "$0")/../../extensions/harness"
echo "toolchain for $(pwd)"
rustc -Vv
cargo -V
