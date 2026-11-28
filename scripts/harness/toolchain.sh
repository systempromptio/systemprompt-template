#!/usr/bin/env bash
# Print the toolchain the desktop shell workspace builds with, and prove the
# part of it rustup does not install: rustc and cargo as rustup resolves them
# from inside extensions/harness, which pins its own channel in
# extensions/harness/rust-toolchain.toml (the window toolkit needs a newer
# compiler than the template's nightly), then, on Linux, the C++ runtime the
# shell links. Run by `just harness-toolchain` and by every job of the Harness
# workflow (Windows, macOS, Linux).
#
# The C++ check exists because the window toolkit links libstdc++ on Linux and
# the compiler driver picks the newest GCC installation on the host whether or
# not that one carries the libstdc++ development files; without them the build
# runs to the final link and fails there. The check compiles one line of C++
# with the C++ compiler cargo's configuration names (CXX) and links it against
# libstdc++ with the linker driver the configuration names for the host
# target, the two programs the workspace build itself runs. macOS and Windows
# skip it: their C++ runtime ships with the platform toolchain rustc already
# requires (Xcode's libc++, the MSVC runtime).
set -euo pipefail
cd "$(dirname "$0")/../../extensions/harness"
workspace="$(pwd)"
echo "toolchain for $workspace"
rustc -Vv
cargo -V

# Print the first value cargo's configuration files give for a key, nearest
# file first: the workspace directory, each parent, then CARGO_HOME.
#   config_value '<section header>' '<key>'
config_value() {
    local dir="$workspace" cargo_home="${CARGO_HOME:-$HOME/.cargo}" file value
    local -a files=()
    while :; do
        files+=("$dir/.cargo/config.toml" "$dir/.cargo/config")
        [ "$dir" != "/" ] || break
        dir="$(dirname "$dir")"
    done
    files+=("$cargo_home/config.toml" "$cargo_home/config")
    for file in "${files[@]}"; do
        [ -f "$file" ] || continue
        value="$(awk -v section="$1" -v key="$2" '
            /^[[:space:]]*\[/ { inside = ($0 == section); next }
            inside {
                line = $0
                sub(/^[[:space:]]*/, "", line)
                if (index(line, key) != 1) next
                rest = substr(line, length(key) + 1)
                if (rest !~ /^[[:space:]]*=/) next
                if (match(rest, /"[^"]*"/)) { print substr(rest, RSTART + 1, RLENGTH - 2); exit }
            }' "$file")"
        if [ -n "$value" ]; then
            printf '%s\n' "$value"
            return 0
        fi
    done
}

# Fail with one line unless the C++ compiler and the linker driver cargo will
# run can build a program against libstdc++.
check_cxx_runtime() {
    local host env_name linker cxx tool selected version package work
    host="$(rustc -vV | sed -n 's/^host: //p')"
    env_name="CARGO_TARGET_$(printf '%s' "$host" | tr 'a-z-' 'A-Z_')_LINKER"
    linker="${!env_name:-}"
    [ -n "$linker" ] || linker="$(config_value "[target.$host]" linker)"
    linker="${linker:-cc}"
    cxx="${CXX:-}"
    [ -n "$cxx" ] || cxx="$(config_value "[env]" CXX)"
    cxx="${cxx:-c++}"

    for tool in "$cxx" "$linker"; do
        command -v "$tool" > /dev/null 2>&1 || {
            echo "harness: toolchain: '$tool' is not installed, and cargo's configuration names it to build the shell's C++ dependencies for $host; install the package that provides it" >&2
            exit 1
        }
    done

    selected="$("$linker" -v 2>&1 | sed -n 's/^Selected GCC installation: //p' | head -n 1)"
    if [ -z "$selected" ]; then
        selected="$(dirname "$("$linker" -print-libgcc-file-name 2> /dev/null)")"
    fi
    version="$(basename "$selected")"
    version="${version%%.*}"
    case "$version" in
        '' | *[!0-9]*) package="the libstdc++ development package of that GCC version (libstdc++-<N>-dev on Debian and Ubuntu, libstdc++-devel on Fedora and RHEL)" ;;
        *) package="libstdc++-$version-dev (Debian, Ubuntu) or libstdc++-devel for GCC $version (Fedora, RHEL)" ;;
    esac

    work="$(mktemp -d)"
    printf '#include <string>\nint main() { return std::string("x").size() == 1 ? 0 : 1; }\n' > "$work/probe.cc"
    if "$cxx" -c "$work/probe.cc" -o "$work/probe.o" > /dev/null 2>&1 \
        && "$linker" "$work/probe.o" -lstdc++ -o "$work/probe" > /dev/null 2>&1; then
        rm -rf "$work"
        echo "C++ runtime: $cxx compiles and $linker links libstdc++ from the GCC installation $selected"
        return 0
    fi
    rm -rf "$work"
    echo "harness: toolchain: $cxx and $linker cannot build against libstdc++ for $host: the selected GCC installation ${selected:-(none found)} has no libstdc++ development files; install $package" >&2
    exit 1
}

case "$(uname -s)" in
    Linux) check_cxx_runtime ;;
    *) echo "C++ runtime: not checked on $(uname -s), where it ships with the platform toolchain" ;;
esac
