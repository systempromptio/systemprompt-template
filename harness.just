# Recipes for the desktop shell workspace (extensions/harness), a cargo
# workspace separate from the root one with its own lockfile and its own
# rust-toolchain.toml. Imported by the justfile; every recipe is named
# harness-*. Each goes through scripts/harness/, which runs cargo from inside
# the workspace so rustup resolves the workspace's toolchain; these recipes
# are not under the build coordinator, which covers the root target/ only.

# Print the toolchain the desktop shell workspace resolves (rustc, cargo)
harness-toolchain:
    bash scripts/harness/toolchain.sh

# Read the desktop shell workspace manifest against its lockfile (no compile)
harness-resolve:
    bash scripts/harness/cargo.sh resolve

# Type-check every shell crate and its tests (the first run builds the window toolkit)
harness-check *ARGS:
    bash scripts/harness/cargo.sh check {{ARGS}}

# Run the shell workspace's tests
harness-test *ARGS:
    bash scripts/harness/cargo.sh test {{ARGS}}

# Lint the shell workspace under its own lint table
harness-clippy *ARGS:
    bash scripts/harness/cargo.sh clippy {{ARGS}}

# Check the shell workspace's formatting against the template's rustfmt.toml
harness-fmt-check:
    bash scripts/harness/cargo.sh fmt-check
