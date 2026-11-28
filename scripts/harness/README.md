# scripts/harness

Scripts for the desktop shell workspace (`extensions/harness`) and the
`harness` flavour. A script lives here when a recipe in `harness.just` and
the `Harness` workflow both need it, so the two cannot drift; anything one
recipe alone needs stays inline in `harness.just`.

| Script | Does |
|---|---|
| `toolchain.sh` | prints `rustc -Vv` and `cargo -V` as resolved from inside `extensions/harness`, then on Linux proves the C++ runtime the shell links: it compiles one line of C++ with the compiler cargo's configuration names (`CXX`) and links it against libstdc++ with the linker driver the configuration names for the host target, and fails with one line naming the missing package and the GCC installation the driver selected. macOS and Windows print that the check does not apply, since their C++ runtime ships with the platform toolchain |
| `cargo.sh` | runs `resolve`, `check`, `test`, `clippy`, `fmt-check`, `build-release` or `deny` from inside `extensions/harness`, so rustup resolves the workspace's own toolchain (`fmt-check` uses the template's, for its `rustfmt.toml`). Every verb but `build-release` covers the shell workspace and then its tests workspace (`extensions/harness/tests`); both build into `extensions/harness/target` unless `CARGO_TARGET_DIR` is set |

Every script here runs on macOS and Linux, and under Git Bash on a Windows
runner: no GNU-only flags.

The flavour-wide scripts stay in `scripts/`: `flavour.py` reads
`flavours/harness/flavour.yaml`, `flavour-build.sh`, `flavour-smoke.sh`,
`flavour-metrics.sh` and `flavour-setup.sh` branch on its `artefact`.
