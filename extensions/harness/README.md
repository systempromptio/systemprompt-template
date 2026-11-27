# extensions/harness

The desktop shell: the window, views, themes, strings and brand of the desktop
app, and the binary that composes them with core's device crates. It is the
`harness` flavour (`flavours/harness/`).

This directory is a cargo workspace of its own. The root `Cargo.toml` lists it
under `exclude`, and it has its own `Cargo.lock`, for one reason: **the
crates here are the only ones in the repository that link the window
toolkit.** The server image never compiles the toolkit, and the desktop build
never compiles the server closure.

## State

Five crates and seven test crates, imported from the harness repository at
`a214511` and moved onto core's crates. `cargo check --workspace
--all-targets` and `cargo test --workspace` pass on macOS against core's
`future` branch; nothing here has been built on Windows or Linux yet, and the
app has not been launched from this tree. `cargo clippy` is clean over the
five crates and red over the test crates, which unwrap, expect and panic
under a lint table that denies all three.

## Members

In dependency order; each depends only on the ones above it:

| Member | Package | Holds |
|---|---|---|
| `brand/` | `systemprompt-harness-brand` | the brand and channel identity: names, app id, URL scheme, default instance, data roots |
| `theme/` | `systemprompt-harness-theme` | design tokens, the brand palettes and their projection onto the toolkit theme |
| `shared/` | `systemprompt-harness-shared` | components, markdown, diff, code blocks, icons, motion; `contract`, the shell extension contract and its registration entry |
| `views/` | `systemprompt-harness-views` | every screen and the frame; the UI model (`state`), the services and their startup (`app`), the keyboard commands and the strings bundle; the Drive, Brain and notifications screens as shell extensions, named in `registered::REGISTERED` |
| `shell/` | `systemprompt-harness-shell` | the launch, the single instance, logging, the main window; the `harness` binary |

`tests/` holds seven crates that test the five through their public API.

The services, the UI model and the strings sit in `views` and not in
`shell` or `shared` because, as imported, they and the views name each
other's types: one crate is the smallest unit that compiles. Splitting them
out is the mount-point work of a later stage, not a move of files.

Shell extensions are named, not collected at link time: each entry of
`views::registered::REGISTERED` is listed under the id constant of its
instance-side member in core's `systemprompt_harness::extension::REGISTERED`.

No UI text literal belongs in a view: every string comes from the strings
bundle. `main` is one line that calls the shell's `run`.

## Configuration

| Concern | Where it lives |
|---|---|
| Toolchain | `rust-toolchain.toml` here (stable 1.99.1): the pinned window toolkit does not build on the template's nightly. rustup resolves it from the working directory, so cargo runs from inside this directory (`scripts/harness/cargo.sh`) |
| Formatting | the template's `rustfmt.toml`, found by rustfmt walking up from here; it needs the template's nightly, which `just harness-fmt-check` selects |
| Clippy thresholds | the template's `clippy.toml`, found by walking up from here |
| Lint levels | `[workspace.lints]` in this `Cargo.toml`: the root table, copied unchanged (cargo reads it per workspace) |
| Supply chain | the template's `deny.toml`, found from `--manifest-path` |
| Lockfile | `Cargo.lock` here |

The `tests/` workspace resolves its configuration the same way. Nothing is
symlinked: a Windows checkout does not materialise symlinks, and this
workspace builds there.

## Core

Core is named the way the root manifest names it: one line per crate in
`[workspace.dependencies]`, a git dependency on one rev of
systemprompt-core. The rev is the head of core's `future` branch, the only
line that carries the device crates (`systemprompt-engine`, `-device`,
`-drive`, `-harness`). `bridge/CORE_REF` holds an older rev for the root
and `tests/` workspaces, so `scripts/core-pin.sh` does not move or check
this workspace yet.

## Recipes

```bash
just harness-toolchain    # rustc and cargo as this workspace resolves them
just harness-resolve      # cargo metadata --locked over this workspace
just harness-check        # cargo check --workspace --all-targets --locked
just harness-test         # cargo test --workspace --locked
just harness-clippy       # cargo clippy --workspace --all-targets --locked
just harness-fmt-check    # rustfmt --check under the template's toolchain
```

`harness.just` at the repository root holds them; `scripts/harness/` holds
the scripts they and the `Harness` workflow share. The first check builds the
window toolkit and takes several minutes.

## Licence

MIT, like the rest of the template. The crates here link core's BUSL-1.1
crates, as `extensions/web` does.
