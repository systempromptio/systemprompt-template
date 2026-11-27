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

An empty scaffold. `Cargo.toml` declares the workspace, its package defaults,
the template's lint table and the inert core override set; `members` is
empty and there is no Rust source. `cargo metadata` resolves it; commands
that need a package (`check`, `build`, `clippy`, `fmt`, `test`) have
nothing to act on until the first member lands.

## What lands here next (S1)

Each directory becomes a workspace member with its contributions in its own
`src/extension.rs`:

| Member | Holds |
|---|---|
| `shared/` | tokens, components, markdown, diff, the strings bundle; the only crate that re-exports toolkit types |
| `shell/` | window, frame, mount points, tray, deep links |
| `views/` | transcript, composer, docks, pickers, settings, sign-in, governance, drive, hosts, marketplace, first run, inbox |
| `theme/` | theme palettes |
| `brand/` | the brand declaration and icons |

Dependencies follow the web extension's shape: `shared` at the bottom;
`shell`, `theme` and `brand` on `shared`; `views` on `shell`. A view
never names a toolkit type outside `shared`'s re-exports, and no view holds
a UI text literal: every string comes from the strings bundle. The binary's
`main` is one line that runs the composition.

The first member also brings what an empty workspace cannot carry: the
`[workspace.dependencies]` table with its core pins, the build profiles, and
the supply-chain, formatting and lint runs over this workspace.

## Configuration

| Concern | Where it lives |
|---|---|
| Toolchain | the template's `rust-toolchain.toml`, found by rustup walking up from here |
| Formatting | the template's `rustfmt.toml`, found the same way |
| Clippy thresholds | the template's `clippy.toml`, found the same way |
| Lint levels | `[workspace.lints]` in this `Cargo.toml`: the root table, copied unchanged (cargo reads it per workspace) |
| Supply chain | the template's `deny.toml`, found from `--manifest-path` |
| Lockfile | `Cargo.lock` here |

The `tests/` workspace resolves its configuration the same way. Nothing is
symlinked: a Windows checkout does not materialise symlinks, and this
workspace builds there.

## Core

The workspace names no core crate yet. When its first member does, core is
pinned here the way the root manifest pins it: git dependencies on the one rev
in `bridge/CORE_REF`, moved by `scripts/core-pin.sh`.

## Recipes

```bash
just harness-toolchain    # rustc and cargo as this workspace resolves them
just harness-resolve      # cargo metadata --locked over this workspace
```

`harness.just` at the repository root holds them; `scripts/harness/` holds
the scripts they and the `Harness` workflow share.

## Licence

MIT, like the rest of the template. The crates here link core's BUSL-1.1
crates, as `extensions/web` does.
