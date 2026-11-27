#!/usr/bin/env bash
# Print the toolchain the desktop shell workspace builds with: rustc and cargo
# as rustup resolves them from inside extensions/harness, which pins its own
# channel in extensions/harness/rust-toolchain.toml (the window toolkit needs a
# newer compiler than the template's nightly). Run by `just harness-toolchain`
# and by every job of the Harness workflow (Windows, macOS, Linux).
set -euo pipefail
cd "$(dirname "$0")/../../extensions/harness"
echo "toolchain for $(pwd)"
rustc -Vv
cargo -V
