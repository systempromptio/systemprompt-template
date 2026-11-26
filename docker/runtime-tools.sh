#!/bin/sh
# Assemble the runtime's tools over a distroless base: busybox (the entrypoint's
# shell and the applets below), jq (secrets.json edits), lsof (core's process
# supervision finds the pids holding a port) and setpriv (drops from root
# after preparing a root-owned volume), each with the shared objects the
# base does not already carry, and the app account. HOME_ROOT receives the
# directories that account owns (/app and /home/app).
#   docker/runtime-tools.sh BASE_ROOT OUT_ROOT HOME_ROOT
set -eu
base="$1"
out="$2"
home="$3"
mkdir -p "$out/bin" "$out/usr/bin" "$out/etc" "$home/home/app"

cp /bin/busybox "$out/bin/busybox"
cp /usr/bin/jq /usr/bin/lsof /usr/bin/setpriv "$out/usr/bin/"
for applet in sh base64 basename cat chmod chown cp cut date dirname echo env \
    false find grep head id kill ln ls mkdir mktemp mv nc od printf ps readlink \
    rm sed sha256sum sleep sort stat tail test timeout touch tr true wc [; do
    ln -s busybox "$out/bin/$applet"
done

for object in $(ldd /bin/busybox /usr/bin/jq /usr/bin/lsof /usr/bin/setpriv | awk '$2 == "=>" { print $3 }' | sort -u); do
    name="$(basename "$object")"
    if [ -n "$(find "$base/lib" "$base/usr/lib" -name "$name" -print -quit)" ]; then continue; fi
    dir="$out/usr/lib/$(basename "$(dirname "$object")")"
    mkdir -p "$dir"
    cp -L "$object" "$dir/$name"
done

cp "$base/etc/passwd" "$base/etc/group" "$out/etc/"
echo 'app:x:1000:1000:app:/home/app:/bin/sh' >> "$out/etc/passwd"
echo 'app:x:1000:' >> "$out/etc/group"

for dir in bin logs data storage/data storage/files/uploads storage/files/images/generated \
    web .systemprompt/profiles/docker; do
    mkdir -p "$home/app/$dir"
done
