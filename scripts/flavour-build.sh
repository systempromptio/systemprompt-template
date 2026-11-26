#!/usr/bin/env bash
# Build one flavour's image: the Dockerfile ARGs come from its flavour.yaml.
#   scripts/flavour-build.sh FLAVOUR [--tag T] [--target STAGE] [--dry-run] [-- DOCKER_BUILD_ARGS...]
# While Cargo.toml patches core to a sibling checkout (scripts/core-patch.sh
# status = live), that checkout is passed as the `core` BuildKit context and
# mounted at the path the patch names (CORE_PATH overrides where it is read
# from). Once core is pinned by git rev there is no patch and no context.
set -euo pipefail
source "$(dirname "$0")/lib/flavour.sh"
flavour="${1:?usage: flavour-build.sh FLAVOUR [--tag T] [--target STAGE] [-- ARGS...]}"
shift
flavour_require "$flavour"
tag="systemprompt-template:$flavour"
target=""
dry_run=false
extra=()
while [ $# -gt 0 ]; do
    case "$1" in
        --tag) tag="${2:?}"; shift 2 ;;
        --target) target="${2:?}"; shift 2 ;;
        --dry-run) dry_run=true; shift ;;
        --) shift; extra=("$@"); break ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done
cd "$FLAVOUR_ROOT"
args=(
    --build-arg "FLAVOUR=$flavour"
    --build-arg "FEATURES=$(flavour_py features "$flavour")"
    --build-arg "BINS=$(flavour_get "$flavour" binaries | tr '\n' ' ' | sed 's/ $//')"
    --build-arg "RUNTIME=$(flavour_get "$flavour" runtime)"
    --build-arg "REQUIRES_AI_KEY=$(flavour_get "$flavour" requires_ai_key)"
    --build-arg "SITE=$(flavour_get "$flavour" site)"
    --tag "$tag"
)
if [ "$(bash scripts/core-patch.sh status)" = live ]; then
    core_rel="$(sed -n 's|^systemprompt = { path = "\(.*\)/systemprompt" }$|\1|p' Cargo.toml | head -1)"
    [ -n "$core_rel" ] || { echo "core patch is live but names no systemprompt path" >&2; exit 1; }
    core_path="${CORE_PATH:-$FLAVOUR_ROOT/$core_rel}"
    [ -d "$core_path/systemprompt" ] || { echo "no core checkout at $core_path (set CORE_PATH)" >&2; exit 1; }
    core_stage="$FLAVOUR_ROOT/target/core-context"
    mkdir -p "$core_stage"
    git -C "$core_path" ls-files -z --cached --others --exclude-standard \
        | rsync -a --delete --ignore-missing-args --from0 --files-from=- "$core_path/" "$core_stage/"
    args+=(--build-context "core=$core_stage" --build-arg "CORE_MOUNT=$(realpath -m "/src/$core_rel")")
    echo "core: patched to $core_path, mounted at $(realpath -m "/src/$core_rel")"
fi
if [ -n "$target" ]; then args+=(--target "$target"); fi
echo "building $tag (flavour $flavour)"
if [ "$dry_run" = true ]; then
    printf '%q ' docker build "${args[@]}" "${extra[@]}" -f Dockerfile .; echo
    exit 0
fi
docker build "${args[@]}" "${extra[@]}" -f Dockerfile .
