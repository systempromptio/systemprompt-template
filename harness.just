# Recipes for the desktop shell (extensions/harness): two cargo workspaces
# separate from the root one, the five shell crates and, under tests/, the
# seven crates that test them, each with its own lockfile, both on the
# toolchain extensions/harness/rust-toolchain.toml pins. Imported by the
# justfile; every recipe is named harness-*. Each goes through
# scripts/harness/cargo.sh, which runs cargo from inside the workspaces so
# rustup resolves that toolchain, covers both (the shell first) and builds
# them into extensions/harness/target; these recipes are not under the build
# coordinator, which covers the root target/ only.

# Print the toolchain the desktop shell workspace resolves (rustc, cargo)
harness-toolchain:
    bash scripts/harness/toolchain.sh

# Read both shell manifests against their lockfiles (no compile)
harness-resolve:
    bash scripts/harness/cargo.sh resolve

# Type-check every shell crate and its tests (the first run builds the window toolkit)
harness-check *ARGS:
    bash scripts/harness/cargo.sh check {{ARGS}}

# Run the shell's tests (extensions/harness/tests, after the shell's own doc tests)
harness-test *ARGS:
    bash scripts/harness/cargo.sh test {{ARGS}}

# Lint both workspaces with warnings as errors: the shell under the root lint table, its tests under the tests table
harness-clippy *ARGS:
    bash scripts/harness/cargo.sh clippy {{ARGS}}

# Check the formatting of both workspaces against the template's rustfmt.toml
harness-fmt-check:
    bash scripts/harness/cargo.sh fmt-check

# Build the release `harness` binary into extensions/harness/target/release
harness-build-release *ARGS:
    bash scripts/harness/cargo.sh build-release {{ARGS}}

# Supply chain over both workspaces (cargo-deny, the template's deny.toml)
harness-deny:
    bash scripts/harness/cargo.sh deny
