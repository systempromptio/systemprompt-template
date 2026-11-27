#!/usr/bin/env bash
# Pins systemprompt-core by git rev. bridge/CORE_REF holds the one rev; every
# core crate the root, tests/ and extensions/mobile/core manifests name (each
# is a separate workspace) is a git dependency on that rev, so core's own
# inter-crate dependencies resolve inside the same git source and each
# lockfile carries exactly one copy of every systemprompt-* crate.
#
# The desktop shell (extensions/harness and its tests workspace,
# extensions/harness/tests) is pinned the same way, and its rev is the one its
# own manifests name. While that rev equals bridge/CORE_REF the shell FOLLOWS:
# moving the pin moves all five workspaces. While it differs (the shell needs
# core's device crates before the root adopts them) the shell is APART: it is
# checked against its own rev, a root move leaves it where it is, and every
# run says so. It follows again from the moment the two revs are equal.
#   scripts/core-pin.sh <rev>          write CORE_REF and every root entry, re-resolve every root lock;
#                                      the shell too when it follows
#   scripts/core-pin.sh --shell <rev>  move the shell's two workspaces only (CORE_REF untouched)
#   scripts/core-pin.sh --check        fail on another rev, a registry or duplicate core crate, or an
#                                      override; a shell that is apart is reported, not failed
#   scripts/core-pin.sh --check-one    --check, and fail while the shell is apart
set -euo pipefail
cd "$(dirname "$0")/.."
url=https://github.com/systempromptio/systemprompt-core
ref_file=bridge/CORE_REF
manifests=(Cargo.toml tests/Cargo.toml extensions/mobile/core/Cargo.toml)
locks=(Cargo.lock tests/Cargo.lock extensions/mobile/core/Cargo.lock)
shell_manifests=(extensions/harness/Cargo.toml extensions/harness/tests/Cargo.toml)
shell_locks=(extensions/harness/Cargo.lock extensions/harness/tests/Cargo.lock)

core_entries() {
    grep -E '^systemprompt[a-z-]* = \{' "$1" | grep -vE 'path = |workspace = true' || true
}

lock_sources() {
    awk '/^\[\[package\]\]/ { name = ""; next }
         /^name = "/ { name = $3; gsub(/"/, "", name); next }
         /^source = "/ && name ~ /^systemprompt/ { print name, $3 }' "$1"
}

# Every distinct rev the shell's manifests name, one per line.
shell_revs() {
    local m
    for m in "${shell_manifests[@]}"; do core_entries "$m"; done \
        | sed -n 's/.*rev = "\([0-9a-f]\{40\}\)".*/\1/p' | sort -u
}

read_ref() {
    local ref
    ref="$(tr -d '[:space:]' < "$ref_file")"
    [[ "$ref" =~ ^[0-9a-f]{40}$ ]] || { echo "core-pin: $ref_file must hold one 40-char core SHA, got '$ref'" >&2; exit 1; }
    printf '%s\n' "$ref"
}

# check_group <rev> <what the rev is> <manifest count> <manifest>... <lock>...
check_group() {
    local ref="$1" what="$2" count="$3" fail=0 m l bad
    shift 3
    for m in "${@:1:$count}"; do
        if grep -qE '^\[patch[.]' "$m"; then
            echo "core-pin: $m carries a [patch] table (a local core override); run scripts/core-patch.sh off" >&2
            fail=1
        fi
        [ -n "$(core_entries "$m")" ] || { echo "core-pin: $m names no core crate" >&2; fail=1; continue; }
        if core_entries "$m" | grep -vF "git = \"$url\", rev = \"$ref\""; then
            echo "core-pin: the entries above in $m are not git dependencies on $url at $what ($ref)" >&2
            fail=1
        fi
    done
    for l in "${@:$((count + 1))}"; do
        bad="$(lock_sources "$l" | grep -vF "\"git+$url?rev=$ref#$ref\"" || true)"
        [ -z "$bad" ] || { printf '%s\n' "$bad" >&2; echo "core-pin: $l resolves the core crates above from another source than $what ($ref)" >&2; fail=1; }
        bad="$(lock_sources "$l" | awk '{ print $1 }' | sort | uniq -d)"
        [ -z "$bad" ] || { printf '%s\n' "$bad" >&2; echo "core-pin: $l holds more than one copy of the core crates above" >&2; fail=1; }
        [ -n "$(lock_sources "$l")" ] || { echo "core-pin: $l resolves no core crate from $ref" >&2; fail=1; }
    done
    return "$fail"
}

check() {
    local one="$1" ref shell fail=0
    ref="$(read_ref)"
    check_group "$ref" "$ref_file" "${#manifests[@]}" "${manifests[@]}" "${locks[@]}" \
        || { echo "core-pin: fix with scripts/core-pin.sh $ref" >&2; fail=1; }
    shell="$(shell_revs)"
    if [ "$(printf '%s\n' "$shell" | grep -c .)" -ne 1 ]; then
        echo "core-pin: the shell manifests (${shell_manifests[*]}) must name exactly one core rev between them, found:" >&2
        printf '%s\n' "${shell:-none}" | sed 's/^/  /' >&2
        echo "core-pin: fix with scripts/core-pin.sh --shell <rev>" >&2
        exit 1
    fi
    check_group "$shell" "the shell's rev" "${#shell_manifests[@]}" "${shell_manifests[@]}" "${shell_locks[@]}" \
        || { echo "core-pin: fix with scripts/core-pin.sh --shell $shell" >&2; fail=1; }
    [ "$fail" -eq 0 ] || exit 1
    if [ "$shell" = "$ref" ]; then
        echo "core-pin: every workspace (${manifests[*]} ${shell_manifests[*]}) resolves one copy of each core crate from $url at $ref"
        return 0
    fi
    echo "core-pin: the root workspaces (${manifests[*]}) resolve one copy of each core crate from $url at $ref"
    {
        echo "core-pin: the desktop shell is APART from $ref_file:"
        echo "  $ref  $ref_file"
        echo "  $shell  ${shell_manifests[*]}"
        echo "  Each side resolves one copy of each core crate from its own rev. The shell follows the root"
        echo "  again once the two are equal: 'scripts/core-pin.sh $shell' moves the root onto the"
        echo "  shell's rev, 'scripts/core-pin.sh --shell $ref' moves the shell onto the root's."
    } >&2
    [ "$one" -eq 0 ] || { echo "core-pin: --check-one requires one rev for every workspace" >&2; exit 1; }
}

resolve_rev() {
    local ref="$1"
    if ! [[ "$ref" =~ ^[0-9a-f]{40}$ ]]; then
        ref="$(git ls-remote "$url" | awk -v r="$ref" '$1 ~ "^"r || $2 == "refs/heads/"r || $2 == "refs/tags/"r { print $1; exit }')"
        [ -n "$ref" ] || { echo "core-pin: $1 names no commit, branch or tag of $url; pass the full SHA" >&2; exit 1; }
    fi
    printf '%s\n' "$ref"
}

# rewrite <rev> <manifest>...
rewrite() {
    local ref="$1" m
    shift
    for m in "$@"; do
        # Why: the address is delimited with "#" because the URL holds
        # slashes, and the edit goes through a copy because `sed -i` takes
        # different arguments on macOS and Linux.
        sed -E "\\#^systemprompt[a-z-]* = \{.*git = \"$(printf '%s' "$url" | sed 's|[.]|[.]|g')\"# s|rev = \"[0-9a-f]{40}\"|rev = \"$ref\"|" "$m" > "$m.pin"
        mv "$m.pin" "$m"
    done
}

# Why: rustup resolves the toolchain from the working directory, not from
# --manifest-path, and the shell pins its own
# (extensions/harness/rust-toolchain.toml), so its locks are re-resolved from
# inside each workspace.
move_shell() {
    local ref="$1" m
    rewrite "$ref" "${shell_manifests[@]}"
    for m in "${shell_manifests[@]}"; do
        (cd "$(dirname "$m")" && cargo metadata --format-version 1 >/dev/null)
    done
}

require_pin() {
    [ "$(bash scripts/core-patch.sh status)" = pin ] || { echo "core-pin: a local core override is on; run scripts/core-patch.sh off first" >&2; exit 1; }
}

case "${1:?usage: scripts/core-pin.sh <rev> | --shell <rev> | --check | --check-one}" in
    --check) check 0 ;;
    --check-one) check 1 ;;
    --shell)
        require_pin
        ref="$(resolve_rev "${2:?usage: scripts/core-pin.sh --shell <rev>}")"
        move_shell "$ref"
        check 0
        ;;
    *)
        require_pin
        ref="$(resolve_rev "$1")"
        was="$(tr -d '[:space:]' < "$ref_file")"
        shell_was="$(shell_revs)"
        printf '%s\n' "$ref" > "$ref_file"
        rewrite "$ref" "${manifests[@]}"
        for m in "${manifests[@]}"; do
            cargo metadata --manifest-path "$m" --format-version 1 >/dev/null
        done
        if [ "$shell_was" = "$was" ]; then
            move_shell "$ref"
        fi
        check 0
        ;;
esac
