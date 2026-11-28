#!/usr/bin/env bash
# tour.sh: the screenshot tour on macOS (S2-C1; core testing/05-evidence.md section 2).
#
# Copies the app bundle packaging/harness/macos/bundle.sh produced into ~/Applications, starts the scripted
# instance (built from a core checkout at the rev extensions/harness/Cargo.toml pins), signs in with
# "harness login --stdin", seeds sessions with "harness run" (seeds.txt), then for each state in states.txt
# reachable by launch or deep link: brings the window to it, waits, captures the window with screencapture and
# checks the capture is not blank and matches the window's size. States no link reaches are written to the
# manual-walk list, never driven blind. The user's data root is moved aside for the tour and put back after.
#
# Writes into --out: <number>-<slug>-<theme>.png, tour.tsv (number|slug|theme|status|file|note, status one of
# captured, failed, blocked, manual) and manual-walk.txt. Exits 1 if any state that should be captured was not.
#
# Usage:
#   extensions/harness/tests/tour/tour.sh --out DIR [--core DIR] [--app PATH.app] [--settle SECONDS] [--dry-run]
#
# --dry-run installs, builds and launches nothing: it reads the lists, prints what each state would do, stands
# the fixture image in for every capture and runs the same checks on it.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../../.." && pwd)"
out=""
core=""
app=""
settle=4
dry_run=0

usage() {
    sed -n '/^# Usage:/,/^# --dry-run/p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --out) [ "$#" -ge 2 ] || usage; out="$2"; shift 2 ;;
        --core) [ "$#" -ge 2 ] || usage; core="$2"; shift 2 ;;
        --app) [ "$#" -ge 2 ] || usage; app="$2"; shift 2 ;;
        --settle) [ "$#" -ge 2 ] || usage; settle="$2"; shift 2 ;;
        --dry-run) dry_run=1; shift ;;
        -h | --help) usage ;;
        *) echo "tour.sh: unknown argument '$1'" >&2; usage ;;
    esac
done
[ -n "$out" ] || usage
mkdir -p "$out"
out="$(cd "$out" && pwd)"

scheme="sp-harness"
app_id="io.systemprompt.harness"
display_name="systemprompt Harness"
data_root="$HOME/Library/Application Support/systemprompt/$display_name"
pinned_rev="$(sed -nE 's/^systemprompt-harness = .*rev = "([0-9a-f]+)".*/\1/p' "$root/extensions/harness/Cargo.toml")"
results="$out/tour.tsv"
manual="$out/manual-walk.txt"
printf 'number|slug|theme|status|file|note\n' >"$results"
: >"$manual"
failures=0
sessions_file="$out/.sessions"
: >"$sessions_file"
backup=""

record() {
    printf '%s|%s|%s|%s|%s|%s\n' "$1" "$2" "$3" "$4" "$5" "$6" >>"$results"
    case "$4" in failed | blocked) failures=$((failures + 1)) ;; esac
    echo "  $1 $2 ($3): $4${6:+ - $6}"
}

# check FILE EXPECTED_WIDTH EXPECTED_HEIGHT: prints nothing when the capture passes, else why not.
check() {
    local file="$1" want_w="$2" want_h="$3" w h size
    [ -s "$file" ] || { echo "no capture written"; return; }
    w="$(sips -g pixelWidth "$file" | awk '/pixelWidth/ {print $2}')"
    h="$(sips -g pixelHeight "$file" | awk '/pixelHeight/ {print $2}')"
    if [ "$w" != "$want_w" ] || [ "$h" != "$want_h" ]; then
        echo "capture is ${w}x${h}, the window ${want_w}x${want_h}"
        return
    fi
    size="$(wc -c <"$file" | tr -d ' ')"
    # Why: a PNG of one flat colour deflates to almost nothing; real window content does not.
    if [ "$size" -lt $((w * h / 400 + 200)) ]; then
        echo "capture looks blank ($size bytes for ${w}x${h})"
    fi
}

session_id() { sed -n "s/^$1|//p" "$sessions_file" | head -n 1; }

expand() {
    local target="$1" name id
    target="${target//\{project\}/$project}"
    while [[ "$target" =~ \{session:([a-z0-9-]+)\} ]]; do
        name="${BASH_REMATCH[1]}"
        id="$(session_id "$name")"
        target="${target//\{session:$name\}/$id}"
    done
    printf '%s' "$target"
}

# ---- the app -------------------------------------------------------------------------------------------------
installed="$HOME/Applications/$display_name.app"
binary="$installed/Contents/MacOS/harness"
pid=""

install_app() {
    [ -n "$app" ] || app="$root/dist/$display_name.app"
    [ -d "$app" ] || { echo "tour.sh: no app bundle at $app (run packaging/harness/macos/bundle.sh)" >&2; return 1; }
    mkdir -p "$HOME/Applications"
    rm -rf "$installed"
    ditto "$app" "$installed"
    /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$installed"
}

set_theme() {
    mkdir -p "$data_root/state"
    printf '{"scheme":"%s"}\n' "$1" >"$data_root/state/ui-settings.json"
}

launch() {
    open -n -a "$installed"
    local tries=0
    pid=""
    while [ -z "$pid" ] && [ "$tries" -lt 30 ]; do
        sleep 1
        pid="$(pgrep -nf "$binary" || true)"
        tries=$((tries + 1))
    done
    [ -n "$pid" ] || return 1
    sleep "$settle"
}

quit_app() {
    osascript -e "tell application id \"$app_id\" to quit" >/dev/null 2>&1 || true
    local tries=0
    while pgrep -f "$binary" >/dev/null && [ "$tries" -lt 10 ]; do
        sleep 1
        tries=$((tries + 1))
    done
    pkill -f "$binary" 2>/dev/null || true
}

# capture NUMBER SLUG THEME
capture() {
    local file="$out/$1-$2-$3.png" bounds x y w h scale problem
    if [ "$dry_run" -eq 1 ]; then
        cp "$here/fixture/screenshot.png" "$file"
        w="$(sips -g pixelWidth "$file" | awk '/pixelWidth/ {print $2}')"
        h="$(sips -g pixelHeight "$file" | awk '/pixelHeight/ {print $2}')"
        problem="$(check "$file" "$w" "$h")"
    else
        osascript -e "tell application id \"$app_id\" to activate" >/dev/null 2>&1 || true
        sleep 1
        bounds="$(osascript -e "tell application \"System Events\" to tell (first process whose unix id is $pid) to get {position, size} of window 1" 2>/dev/null | tr -d ' ')" || bounds=""
        if [ -z "$bounds" ]; then
            record "$1" "$2" "$3" failed "" "no window found (grant Accessibility to the terminal)"
            return
        fi
        IFS=, read -r x y w h <<<"$bounds"
        screencapture -x -o -R "$x,$y,$w,$h" "$file"
        scale="$(sips -g pixelWidth "$file" | awk -v w="$w" '/pixelWidth/ {printf "%d", $2 / w}')"
        [ "$scale" -ge 1 ] 2>/dev/null || scale=1
        problem="$(check "$file" $((w * scale)) $((h * scale)))"
    fi
    if [ -n "$problem" ]; then
        record "$1" "$2" "$3" failed "$(basename "$file")" "$problem"
    else
        record "$1" "$2" "$3" captured "$(basename "$file")" ""
    fi
}

open_link() {
    if [ "$dry_run" -eq 1 ]; then
        echo "    would open $scheme://$1"
    else
        open "$scheme://$1"
        sleep "$settle"
    fi
}

# ---- the scripted instance and sign-in -------------------------------------------------------------------------
instance_pid=""
signed_in=0
blocked_reason=""
project="$out/.workspace"

cleanup() {
    [ "$dry_run" -eq 1 ] && return
    quit_app
    if [ -n "$instance_pid" ]; then kill "$instance_pid" 2>/dev/null || true; fi
    if [ -n "$backup" ] && [ -d "$backup" ]; then
        rm -rf "$data_root"
        mv "$backup" "$data_root"
    fi
}
trap cleanup EXIT

sign_in() {
    rm -rf "$project"
    cp -R "$here/workspace" "$project"
    if [ "$dry_run" -eq 1 ]; then
        echo "  would start the scripted instance from core at $pinned_rev and sign in with login --stdin"
        signed_in=1
        return
    fi
    if ! grep -a -q -- 'login --stdin' "$binary"; then
        blocked_reason="this harness binary does not dispatch login/run (core command module not at the pinned rev)"
        return
    fi
    if [ -z "$core" ] || [ "$(git -C "$core" rev-parse HEAD 2>/dev/null)" != "$pinned_rev" ]; then
        blocked_reason="no core checkout at $pinned_rev (--core) to build the scripted instance from"
        return
    fi
    local target="$out/.instance-target" line base_url pat
    cargo build --release --manifest-path "$core/crates/tests/Cargo.toml" -p systemprompt-scripted-instance \
        --target-dir "$target" >"$out/instance-build.log" 2>&1 || { blocked_reason="scripted instance did not build (instance-build.log)"; return; }
    rm -f "$out/.instance-out"
    mkfifo "$out/.instance-out"
    "$target/release/systemprompt-scripted-instance" --script "$here/instance/turns.json" \
        --messages-model tour-model >"$out/.instance-out" 2>"$out/instance.log" &
    instance_pid=$!
    exec 3<"$out/.instance-out"
    read -r line <&3
    base_url="$(printf '%s' "$line" | sed -nE 's/.*"base_url":"([^"]+)".*/\1/p')"
    pat="$(printf '%s' "$line" | sed -nE 's/.*"pat":"([^"]+)".*/\1/p')"
    if ! printf '%s\n' "$pat" | "$binary" login --stdin --instance "$base_url" >"$out/login.jsonl" 2>&1; then
        blocked_reason="login --stdin failed (login.jsonl)"
        return
    fi
    signed_in=1
    local name mode prompt id
    while IFS='|' read -r name mode prompt; do
        case "$name" in '' | '#'*) continue ;; esac
        "$binary" run --workspace "$project" --prompt "$prompt" --permission-mode "$mode" --timeout 120 \
            >"$out/seed-$name.jsonl" 2>&1 || true
        id="$(sed -nE 's/^\{"type":"started","session_id":"([^"]+)".*/\1/p' "$out/seed-$name.jsonl" | head -n 1)"
        if [ -n "$id" ]; then printf '%s|%s\n' "$name" "$id" >>"$sessions_file"; fi
    done <"$here/seeds.txt"
}

# ---- the tour ------------------------------------------------------------------------------------------------
states() { grep -v -e '^#' -e '^$' "$here/states.txt"; }
on_theme() { [[ ",$1," == *",$2,"* ]]; }

mode_note=""
[ "$dry_run" -eq 1 ] && mode_note=", dry run"
echo "tour: $(states | wc -l | tr -d ' ') states, out $out$mode_note"
if [ "$dry_run" -eq 0 ]; then
    install_app
    quit_app
    if [ -d "$data_root" ]; then
        backup="$data_root.tour-backup-$(date +%Y%m%d%H%M%S)"
        mv "$data_root" "$backup"
    fi
fi

while IFS='|' read -r number slug title themes reach _target _needs; do
    if [ "$reach" = manual ]; then
        printf '%s %s [%s]\n' "$number" "$title" "$themes" >>"$manual"
        record "$number" "$slug" "$themes" manual "" "no deep link or command reaches it"
    fi
done < <(states)

# Signed out: the app launched on the fresh data root.
for theme in light dark; do
    while IFS='|' read -r number slug _title themes reach _target _needs; do
        if [ "$reach" != launch ] || ! on_theme "$themes" "$theme"; then continue; fi
        if [ "$dry_run" -eq 1 ]; then
            echo "    would launch $display_name signed out ($theme)"
        else
            set_theme "$theme"
            launch || { record "$number" "$slug" "$theme" failed "" "the app did not start"; continue; }
        fi
        capture "$number" "$slug" "$theme"
        [ "$dry_run" -eq 1 ] || quit_app
    done < <(states)
done

sign_in
for theme in light dark; do
    launched=0
    while IFS='|' read -r number slug _title themes reach target needs; do
        if [ "$reach" != link ] || ! on_theme "$themes" "$theme"; then continue; fi
        if [ "$needs" != - ] && [ "$signed_in" -eq 0 ]; then
            record "$number" "$slug" "$theme" blocked "" "$blocked_reason"
            continue
        fi
        seed="${needs#seed:}"
        if [[ "$needs" == seed:* ]]; then
            [ "$dry_run" -eq 1 ] && printf '%s|ses_dry_run_%s\n' "$seed" "$seed" >>"$sessions_file"
            if [ -z "$(session_id "$seed")" ]; then
                record "$number" "$slug" "$theme" blocked "" "seed $seed made no session (seed-$seed.jsonl)"
                continue
            fi
        fi
        if [ "$launched" -eq 0 ] && [ "$dry_run" -eq 0 ]; then
            set_theme "$theme"
            launch || { record "$number" "$slug" "$theme" failed "" "the app did not start"; continue; }
            launched=1
        fi
        open_link "$(expand "$target")"
        capture "$number" "$slug" "$theme"
    done < <(states)
    [ "$launched" -eq 1 ] && quit_app
done

captured="$(grep -c '|captured|' "$results" || true)"
echo "tour: $captured captured, $failures failed or blocked, $(wc -l <"$manual" | tr -d ' ') on the manual walk"
[ "$failures" -eq 0 ]
