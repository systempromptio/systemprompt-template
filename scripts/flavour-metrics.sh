#!/usr/bin/env bash
# Measure flavours: image, deploy and developer-loop numbers.
#   scripts/flavour-metrics.sh FLAVOUR|all [--image] [--dev] [--bench]
#       [--compare OLD.json] [--keep] [--no-clean]
# --image  stripped binary bytes, image bytes (uncompressed, gzip), layers,
#          crates in the dependency closure, clean and cached image build
#          seconds, migrate seconds on a fresh database, boot-to-healthy
#          seconds and idle RSS 30 s after healthy (the default mode)
# --no-clean  with --image: skip the clean compile (keeps the previous
#          build.clean_seconds) and time only the cached image build
# --bench  with --image: hey p50/p99 against /api/v1/health
# --dev    cargo check and clippy (--all-targets -D warnings), each clean (a
#          fresh CARGO_TARGET_DIR) and incremental after touching src/lib.rs and
#          extensions/web/admin/src/lib.rs; tests/ workspace compile and run
#          seconds; the 15 slowest units of a clean `cargo build`. Every cargo
#          command runs alone; FLAVOUR_METRICS_JOBS sets -j. Its target dirs
#          (target/flavour-metrics/dev-*) are removed afterwards unless --keep.
# --compare OLD.json  print before/after deltas against the new (or, with no
#          measuring mode, the existing) target/flavour-metrics/<flavour>.json
# Writes target/flavour-metrics/<flavour>.json and target/flavour-metrics/table.md.
set -euo pipefail
source "$(dirname "$0")/lib/flavour.sh"
selection="${1:?usage: flavour-metrics.sh FLAVOUR|all [--image] [--dev] [--bench] [--compare OLD.json] [--keep] [--no-clean]}"
shift
do_image=false do_dev=false do_bench=false keep=false compare="" clean=true
while [ $# -gt 0 ]; do
    case "$1" in
        --image) do_image=true; shift ;;
        --dev) do_dev=true; shift ;;
        --bench) do_bench=true; do_image=true; shift ;;
        --keep) keep=true; shift ;;
        --no-clean) clean=false; shift ;;
        --compare) compare="${2:?}"; shift 2 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done
if [ "$do_image" = false ] && [ "$do_dev" = false ] && [ -z "$compare" ]; then do_image=true; fi
cd "$FLAVOUR_ROOT"
out="$FLAVOUR_ROOT/target/flavour-metrics"
mkdir -p "$out"
if [ "$selection" = all ]; then mapfile -t flavours < <(flavour_py list); else flavour_require "$selection"; flavours=("$selection"); fi
jobs=()
if [ -n "${FLAVOUR_METRICS_JOBS:-}" ]; then jobs=(-j "$FLAVOUR_METRICS_JOBS"); fi

now() { date +%s.%N; }
since() { awk -v end="$(now)" -v start="$1" 'BEGIN { printf "%.2f", end - start }'; }
put() { printf '%s\t%s\n' "$1" "$2" >> "$kv"; }
put_str() { put "$1" "$(jq -Rn --arg v "$2" '$v')"; }
timed() {
    local key="$1"; shift
    local start; start="$(now)"
    if "$@" > "$work/last.log" 2>&1; then
        put "$key" "$(since "$start")"
    else
        put "$key" null
        put_str "$key.error" "$(tail -5 "$work/last.log")"
        echo "  $key failed (see the JSON)" >&2
    fi
}
to_bytes() {
    awk '{ v=$1; u=$1; gsub(/[0-9.]/, "", u); gsub(/[^0-9.]/, "", v);
           m = (u=="KiB"||u=="kB"||u=="KB") ? 1024 : (u=="MiB"||u=="MB") ? 1048576 : (u=="GiB"||u=="GB") ? 1073741824 : 1;
           printf "%d", v*m }'
}
core_rev() {
    if [ "$(bash scripts/core-patch.sh status)" = live ]; then
        local rel; rel="$(sed -n 's|^systemprompt = { path = "\(.*\)/systemprompt" }$|\1|p' Cargo.toml | head -1)"
        git -C "${CORE_PATH:-$FLAVOUR_ROOT/$rel}" rev-parse HEAD 2>/dev/null || echo unknown
    else
        tr -d '[:space:]' < bridge/CORE_REF
    fi
}

measure_image() {
    local flavour="$1" image="systemprompt-template:$1" features
    features="$(flavour_py features "$flavour")"
    if [ "$clean" = true ]; then
        echo "  image build (clean compile: the flavour's target cache is reset)"
        timed build.reset bash scripts/flavour-build.sh "$flavour" --target cache-reset -- --no-cache
        timed build.clean_seconds bash scripts/flavour-build.sh "$flavour" --tag "$image" -- --no-cache
    fi
    timed build.cached_seconds bash scripts/flavour-build.sh "$flavour" --tag "$image"
    local total=0 size
    while IFS= read -r bin; do
        [ -n "$bin" ] || continue
        size="$(docker run --rm --entrypoint stat "$image" -c %s "/app/bin/$bin")"
        put "binaries.$bin" "$size"
        total=$((total + size))
    done < <(flavour_get "$flavour" binaries)
    put binary_bytes "$total"
    put image.bytes "$(docker image inspect -f '{{.Size}}' "$image")"
    put image.gzip_bytes "$(docker save "$image" | gzip -c | wc -c)"
    put image.layers "$(docker image inspect -f '{{len .RootFS.Layers}}' "$image")"
    if cargo tree --locked --no-default-features --features "$features" -e normal,build --prefix none \
        > "$work/tree.txt" 2> "$work/last.log"; then
        put crates "$(sed 's/ (\*)$//; s/ (proc-macro)//' "$work/tree.txt" | sort -u | wc -l)"
    else
        put crates null
        put_str crates_error "$(tail -5 "$work/last.log")"
    fi

    local project="sp-metrics-$flavour" env_file="$work/metrics.env" ai_key=""
    if [ "$(flavour_get "$flavour" requires_ai_key)" = true ]; then ai_key=sk-ant-metrics-placeholder; fi
    printf 'ADMIN_EMAIL=metrics@example.invalid\nPOSTGRES_PASSWORD=metrics\nANTHROPIC_API_KEY=%s\nHTTP_PORT=0\nFLAVOUR=%s\nIMAGE=%s\n' \
        "$ai_key" "$flavour" "$image" > "$env_file"
    local base_args=(--env-file "$env_file" -p "$project" -f docker-compose.yml)
    flavour_isolated_override "$work/isolated.yml" "$image" app "${base_args[@]}"
    local compose=(docker compose "${base_args[@]}" -f "$work/isolated.yml")
    "${compose[@]}" up -d --wait postgres > /dev/null
    local start; start="$(now)"
    "${compose[@]}" up -d app > /dev/null
    local port base healthy=false
    port="$("${compose[@]}" port app 8080 | head -1 | sed 's/.*://')"
    base="http://localhost:$port"
    for _ in $(seq 1 300); do
        if [ "$(docker inspect -f '{{.RestartCount}}' "$("${compose[@]}" ps -q app)")" -gt 0 ]; then break; fi
        if curl -fs --max-time 2 "$base/api/v1/health" 2>/dev/null | jq -e '.status == "healthy"' > /dev/null 2>&1; then healthy=true; break; fi
        sleep 1
    done
    if [ "$healthy" = true ]; then
        put deploy.boot_to_healthy_seconds "$(since "$start")"
        "${compose[@]}" exec -T postgres psql -U systemprompt -d postgres -qc 'CREATE DATABASE sp_metrics_fresh' > /dev/null
        timed deploy.migrate_fresh_seconds "${compose[@]}" exec -T app /app/bin/systemprompt \
            --database-url postgres://systemprompt:metrics@postgres:5432/sp_metrics_fresh infra db migrate
        local second; second="$(now)"
        "${compose[@]}" restart app > /dev/null
        healthy=false
        for _ in $(seq 1 300); do
            # Why: the ephemeral host port is reassigned when the container restarts.
            port="$("${compose[@]}" port app 8080 2>/dev/null | head -1 | sed 's/.*://')"
            base="http://localhost:$port"
            if [ -n "$port" ] && curl -fs --max-time 2 "$base/api/v1/health" 2>/dev/null | jq -e '.status == "healthy"' > /dev/null 2>&1; then healthy=true; break; fi
            sleep 1
        done
        if [ "$healthy" = true ]; then put deploy.second_boot_to_healthy_seconds "$(since "$second")"; else put deploy.second_boot_to_healthy_seconds null; fi
        sleep 30
        put deploy.idle_rss_bytes "$(docker stats --no-stream --format '{{.MemUsage}}' "$("${compose[@]}" ps -q app)" | cut -d/ -f1 | to_bytes)"
        if [ "$do_bench" = true ]; then
            if command -v hey > /dev/null; then
                hey -n 2000 -c 50 "$base/api/v1/health" > "$work/hey.txt"
                put bench.p50_seconds "$(awk '/ 50% in/{print $3}' "$work/hey.txt")"
                put bench.p99_seconds "$(awk '/ 99% in/{print $3}' "$work/hey.txt")"
                put bench.requests_per_second "$(awk '/Requests\/sec/{print $2}' "$work/hey.txt")"
            else
                put_str bench.error "hey is not installed"
            fi
        fi
    else
        put deploy.boot_to_healthy_seconds null
        put_str deploy.error "not healthy: $("${compose[@]}" logs --no-color --no-log-prefix app 2>&1 | sed 's/\x1b\[[0-9;]*[A-Za-z]//g' | grep -m1 -E 'Error|ERROR' | cut -c1-400)"
    fi
    "${compose[@]}" down -v --remove-orphans > /dev/null 2>&1 || true
}

measure_dev() {
    local flavour="$1" features dev="$out/dev-$1"
    features="$(flavour_py features "$flavour")"
    local cargo=(env SQLX_OFFLINE=true cargo)
    local args=(--workspace --locked --no-default-features --features "$features" "${jobs[@]}")
    local deep=extensions/web/admin/src/lib.rs
    rm -rf "$dev"
    for tool in check clippy; do
        local extra=() dir="$dev/$tool"
        if [ "$tool" = clippy ]; then extra=(--all-targets -- -D warnings); fi
        echo "  cargo $tool (clean, then incremental)"
        timed "dev.$tool.clean_seconds" env CARGO_TARGET_DIR="$dir" "${cargo[@]}" "$tool" "${args[@]}" "${extra[@]}"
        touch src/lib.rs
        timed "dev.$tool.touch_root_seconds" env CARGO_TARGET_DIR="$dir" "${cargo[@]}" "$tool" "${args[@]}" "${extra[@]}"
        touch "$deep"
        timed "dev.$tool.touch_admin_seconds" env CARGO_TARGET_DIR="$dir" "${cargo[@]}" "$tool" "${args[@]}" "${extra[@]}"
    done
    echo "  tests/ workspace (compile, then run)"
    timed dev.tests.compile_seconds env CARGO_TARGET_DIR="$dev/tests" "${cargo[@]}" test --no-run --locked --manifest-path tests/Cargo.toml --workspace "${jobs[@]}"
    timed dev.tests.run_seconds env CARGO_TARGET_DIR="$dev/tests" "${cargo[@]}" test --locked --manifest-path tests/Cargo.toml --workspace "${jobs[@]}"
    echo "  cargo build --timings (clean)"
    local start; start="$(now)"
    if env CARGO_TARGET_DIR="$dev/build" "${cargo[@]}" build "${args[@]}" --timings=json -Z unstable-options \
        --message-format json > "$work/timings.jsonl" 2> "$work/last.log"; then
        put dev.build.clean_seconds "$(since "$start")"
        put dev.build.top_units "$(jq -s -c '[.[] | select(.reason == "timing-info")
            | {unit: "\(.package_id | split("#") | last) \(.target.name)", seconds: (.duration * 100 | round / 100)}]
            | sort_by(-.seconds) | .[:15]' "$work/timings.jsonl")"
    else
        put dev.build.clean_seconds null
        put_str dev.build.error "$(tail -5 "$work/last.log")"
    fi
    if [ "$keep" = false ]; then rm -rf "$dev"; fi
}

write_json() {
    local flavour="$1" file="$out/$1.json"
    local previous='{}'
    if [ -f "$file" ]; then previous="$(cat "$file")"; fi
    jq -n --argjson previous "$previous" --rawfile kv "$kv" '
        reduce ($kv | split("\n")[] | select(length > 0) | split("\t")) as [$key, $value]
            ($previous; setpath($key | split("."); ($value | fromjson)))' > "$file.tmp"
    mv "$file.tmp" "$file"
}

write_table() {
    local rows=()
    for flavour in $(flavour_py list); do [ -f "$out/$flavour.json" ] && rows+=("$out/$flavour.json"); done
    [ ${#rows[@]} -gt 0 ] || return 0
    jq -s -r '
        def mb: if . == null then "–" else (. / 1048576 * 10 | round / 10 | tostring) + " MiB" end;
        def s: if . == null then "–" else (. | tostring) + " s" end;
        "| flavour | binary | image | image gz | layers | crates | build clean | build cached | migrate | boot | idle RSS | check clean | check incr (admin) | clippy clean | clippy incr (admin) | tests compile |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|",
        (.[] | "| \(.flavour) | \(.binary_bytes | mb) | \(.image.bytes | mb) | \(.image.gzip_bytes | mb) | \(.image.layers // "–") | \(.crates // "–") | \(.build.clean_seconds | s) | \(.build.cached_seconds | s) | \(.deploy.migrate_fresh_seconds | s) | \(.deploy.boot_to_healthy_seconds | s) | \(.deploy.idle_rss_bytes | mb) | \(.dev.check.clean_seconds | s) | \(.dev.check.touch_admin_seconds | s) | \(.dev.clippy.clean_seconds | s) | \(.dev.clippy.touch_admin_seconds | s) | \(.dev.tests.compile_seconds | s) |"),
        "",
        (.[0].stamp | "core \(.core_rev // "?"), template \(.template_sha // "?"), \(.rustc // "?"), \(.host_cores // "?") cores, \(.host_ram_bytes // 0 | . / 1073741824 | round) GiB RAM, \(.measured_at // "?")")
        ' "${rows[@]}" > "$out/table.md"
}

print_compare() {
    local old="$1" new="$2"
    jq -r -n --slurpfile old "$old" --slurpfile new "$new" '
        [$new[0] | paths(numbers)] as $paths
        | ["metric", "before", "after", "delta", "%"],
          ($paths[] as $p
            | ($old[0] | getpath($p)) as $before
            | ($new[0] | getpath($p)) as $after
            | select($p[0] != "stamp")
            | [($p | map(tostring) | join(".")),
               ($before // "–" | tostring), ($after | tostring),
               (if $before then ($after - $before) * 100 | round / 100 | tostring else "–" end),
               (if $before and $before != 0 then (($after - $before) / $before * 1000 | round / 10 | tostring) + "%" else "–" end)])
        | @tsv' | column -t -s $'\t'
}

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
for flavour in "${flavours[@]}"; do
    kv="$work/$flavour.kv"
    : > "$kv"
    if [ "$do_image" = true ] || [ "$do_dev" = true ]; then
        echo "== metrics: $flavour"
        put_str flavour "$flavour"
        put_str stamp.core_rev "$(core_rev)"
        put_str stamp.template_sha "$(git rev-parse HEAD)"
        put_str stamp.rustc "$(rustc -V)"
        put stamp.host_cores "$(nproc)"
        put stamp.host_ram_bytes "$(awk '/MemTotal/{print $2 * 1024}' /proc/meminfo)"
        put_str stamp.measured_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        put_str stamp.jobs "${FLAVOUR_METRICS_JOBS:-default}"
        if [ "$do_image" = true ]; then measure_image "$flavour"; fi
        if [ "$do_dev" = true ]; then measure_dev "$flavour"; fi
        write_json "$flavour"
        echo "   wrote target/flavour-metrics/$flavour.json"
    fi
    if [ -n "$compare" ]; then
        [ -f "$out/$flavour.json" ] || { echo "no target/flavour-metrics/$flavour.json to compare" >&2; exit 1; }
        echo "== $flavour: $compare -> target/flavour-metrics/$flavour.json"
        print_compare "$compare" "$out/$flavour.json"
    fi
done
write_table
if [ -f "$out/table.md" ]; then echo "table: target/flavour-metrics/table.md"; fi
