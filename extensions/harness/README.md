# extensions/harness

The desktop shell: the window, views, themes, strings and brand of the desktop
app, and the binary that composes them with core's device crates. It is the
`harness` flavour (`flavours/harness/`).

This directory is a cargo workspace of its own. The root `Cargo.toml` lists it
under `exclude`, and it has its own `Cargo.lock`, for one reason: **the
crates here are the only ones in the repository that link the window
toolkit.** The server image never compiles the toolkit, and the desktop build
never compiles the server closure. `tests/` is a second workspace, excluded
from this one the way the root excludes its own `tests/`, with its own
`Cargo.lock` and the template's tests lint table.

## State

Five crates and, in the `tests/` workspace, seven test crates, imported from
the harness repository at `a214511` and moved onto core's crates at core
`8bebe160` (`future`). On macOS, over both workspaces: `cargo check
--all-targets` passes, the 269 tests pass, and `cargo clippy --all-targets
-- -D warnings` is clean (the five crates under the root lint table, the
test crates under the tests table). Formatting and the template's source
gates pass. `cargo deny` passes licences, bans and sources and fails
advisories on one finding, RUSTSEC-2026-0459: the SQLite that core's engine
bundles through `rusqlite` 0.39; the fix is a newer `rusqlite` in core.

Nothing here has been built on Windows or Linux yet: the `Harness` workflow
(`.github/workflows/harness.yml`) does that on the three systems and has
not run on this state. The app has not been launched from this tree, and no
release build or installer has been made from it. The binary has no version
argument and no version resource, so the workflow reads the version back
from the installer.

## Members

In dependency order; each depends only on the ones above it:

| Member | Package | Holds |
|---|---|---|
| `brand/` | `systemprompt-harness-brand` | the brand and channel identity: names, app id, URL scheme, default instance, data roots |
| `theme/` | `systemprompt-harness-theme` | design tokens, the brand palettes and their projection onto the toolkit theme |
| `shared/` | `systemprompt-harness-shared` | components, markdown, diff, code blocks, icons, motion; `contract`, the shell extension contract and its registration entry |
| `views/` | `systemprompt-harness-views` | every screen and the frame; the UI model (`state`), the services and their startup (`app`), the keyboard commands and the strings bundle; the Drive, Brain and notifications screens as shell extensions, named in `registered::REGISTERED` |
| `shell/` | `systemprompt-harness-shell` | the launch, the single instance, logging, the main window; the `harness` binary |

`tests/` holds seven crates that test the five through their public API,
as a separate workspace (`tests/Cargo.toml`).

The services, the UI model and the strings sit in `views` and not in
`shell` or `shared` because, as imported, they and the views name each
other's types: one crate is the smallest unit that compiles. Splitting them
out is the mount-point work of a later stage, not a move of files.

Shell extensions are named, not collected at link time: each entry of
`views::registered::REGISTERED` is listed under the id constant of its
instance-side member in core's `systemprompt_harness::members::REGISTERED`.

No UI text literal belongs in a view: every string comes from the strings
bundle. `main` is one line that calls the shell's `run`.

## Configuration

| Concern | Where it lives |
|---|---|
| Toolchain | `rust-toolchain.toml` here (stable 1.99.1): the pinned window toolkit does not build on the template's nightly. rustup resolves it from the working directory, so cargo runs from inside this directory (`scripts/harness/cargo.sh`) |
| Formatting | the template's `rustfmt.toml`, found by rustfmt walking up from here; it needs the template's nightly, which `just harness-fmt-check` selects |
| Clippy thresholds | the template's `clippy.toml`, found by walking up from here |
| Lint levels | `[workspace.lints]` in this `Cargo.toml`: the root table, copied unchanged (cargo reads it per workspace). `tests/Cargo.toml` carries the template's tests table (`tests/Cargo.toml` at the root), copied unchanged |
| Supply chain | the template's `deny.toml`, found by walking up; it holds the window toolkit's licence exceptions and lists the unmaintained crates the toolkit brings. Both workspaces are in the justfile's `workspaces` (`just deny`, `just machete`, `just audit`) |
| Lockfile | `Cargo.lock` here, and `tests/Cargo.lock` |
| Build output | `target/` here for both workspaces (`scripts/harness/cargo.sh` sets it; the packagers read `target/release`) |

The `tests/` workspace resolves its configuration the same way, walking up
through this directory. Nothing is symlinked: a Windows checkout does not
materialise symlinks, and this workspace builds there.

## Core

Core is named the way the root manifest names it: one line per crate in
`[workspace.dependencies]`, a git dependency on one rev of
systemprompt-core. `tests/Cargo.toml` names the same crates at the same
rev.

The rev is on core's `future` branch and is ahead of the one
`bridge/CORE_REF` holds for the root, `tests/` and `extensions/mobile/core`.
`scripts/core-pin.sh` handles both states:

- **Apart** (now): `--check` holds the two shell manifests to one rev between
  them and each shell lockfile to one copy of every core crate from that rev,
  holds the root workspaces to `bridge/CORE_REF` as before, prints both revs
  and passes. `core-pin.sh <rev>` moves the root and leaves the shell;
  `core-pin.sh --shell <rev>` moves the shell alone. `--check-one` fails
  while they differ.
- **Following**: once the shell's rev equals `bridge/CORE_REF`,
  `core-pin.sh <rev>` (`just core-pin`) moves all five workspaces.

`scripts/core-patch.sh on <core checkout>` switches both shell workspaces to
the checkout together with the root's.

## Recipes

```bash
just harness-toolchain      # rustc and cargo as this workspace resolves them
just harness-resolve        # cargo metadata --locked
just harness-check          # cargo check --workspace --all-targets --locked
just harness-test           # cargo test --workspace --locked
just harness-clippy         # cargo clippy --workspace --all-targets --locked -- -D warnings
just harness-fmt-check      # rustfmt --check under the template's toolchain
just harness-build-release  # the release `harness` binary, target/release here
just harness-deny           # cargo deny check
```

Every recipe but `harness-build-release` covers this workspace and then
`tests/`. `harness.just` at the repository root holds them;
`scripts/harness/cargo.sh` is what they and the `Harness` workflow share.
The first check builds the window toolkit and takes several minutes.

## Licence

MIT, like the rest of the template. The crates here link core's BUSL-1.1
crates, as `extensions/web` does.
