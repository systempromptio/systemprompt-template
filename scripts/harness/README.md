# scripts/harness

Scripts for the desktop shell workspace (`extensions/harness`) and the
`harness` flavour. A script lives here when a recipe in `harness.just` and
the `Harness` workflow both need it, so the two cannot drift; anything one
recipe alone needs stays inline in `harness.just`.

| Script | Does |
|---|---|
| `toolchain.sh` | prints `rustc -Vv` and `cargo -V` as resolved from inside `extensions/harness` |

Every script here runs on macOS and Linux, and under Git Bash on a Windows
runner: no GNU-only flags.

The flavour-wide scripts stay in `scripts/`: `flavour.py` reads
`flavours/harness/flavour.yaml`, `flavour-build.sh`, `flavour-smoke.sh`,
`flavour-metrics.sh` and `flavour-setup.sh` branch on its `artefact`, and
`core-patch.sh` switches the core override in the root and shell manifests
together.
