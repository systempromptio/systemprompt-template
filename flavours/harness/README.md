# harness

The desktop app: a native window on Windows, macOS and Linux, shipped as
installers. It is the one flavour that is not a server image, which
`artefact: desktop` in `flavour.yaml` says; every other flavour is
`artefact: image`, the default.

It is built from `extensions/harness`, a cargo workspace separate from the
root one with its own `Cargo.lock`, so the server images never compile the
window toolkit and the desktop build never compiles the server closure. It
names no root cargo feature, ships no `services/` subtree and has no profile
overlay: `flavour.yaml` holds `name`, `description`, `artefact` and
`workspace`, and `scripts/flavour.py check` refuses the image keys here.

## What works today

The workspace is an empty scaffold: it has no members, so there is nothing to
build, package or launch yet. The shell, views, themes and brand crates and
the packaging arrive in the next stage of the harness programme (S1).

```bash
just flavours                 # lists harness beside the image flavours
just harness-toolchain        # the toolchain the shell workspace resolves
just harness-resolve          # resolve the workspace against its lockfile
```

The flavour recipes accept the name and say so rather than failing:
`just docker-build harness`, `just smoke harness`, `just metrics harness`
and `just setup harness` each print that the desktop build arrives in S1 and
exit 0. `just metrics all` and the `Flavours` workflow measure the image
flavours only.

The `Harness` workflow (`.github/workflows/harness.yml`) runs on Windows,
macOS and Linux. For now each job checks out and prints the toolchain; it
builds nothing and stores no artefact.

## What it becomes

A build of this flavour produces one installer per system from the workspace's
binary, and its smoke test launches the packaged app against a mock instance
and reaches the sign-in gate. `scripts/flavour-build.sh` and
`scripts/flavour-smoke.sh` already branch on `artefact`; the desktop branch
is where those steps land.
