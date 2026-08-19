#!/usr/bin/env bash
# Package everything a self-host deployment needs into one tarball.
#
# Usage: bundle-profile.sh <profile>
#
# The manifest mirrors .systemprompt/profiles/production/docker/Dockerfile —
# the authoritative list of what a running instance requires: the release
# binary, MCP server binaries, storage/, web/dist/, MCP extension assets,
# services/, and the chosen profile. Output: dist/systemprompt-<profile>.tar.gz
# with an UNPACK.md describing first-run steps on the target machine.
set -euo pipefail

PROFILE="${1:?usage: bundle-profile.sh <profile>}"
DIR=".systemprompt/profiles/$PROFILE"
[ -d "$DIR" ] || { echo "ERROR: $DIR does not exist"; exit 1; }

MISSING=0
for p in target/release/systemprompt web/dist storage services; do
    [ -e "$p" ] || { echo "ERROR: $p missing — run: just build-all"; MISSING=1; }
done
[ "$MISSING" -eq 0 ] || exit 1

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
ROOT="$STAGE/systemprompt-$PROFILE"
mkdir -p "$ROOT/bin" "$ROOT/services/profiles"

cp target/release/systemprompt "$ROOT/bin/"
for mcp in target/release/systemprompt-mcp-*; do
    [ -x "$mcp" ] && cp "$mcp" "$ROOT/bin/"
done
cp -R storage "$ROOT/storage"
mkdir -p "$ROOT/web"
cp -R web/dist "$ROOT/web/dist"
if [ -d extensions/mcp ]; then
    mkdir -p "$ROOT/extensions"
    cp -R extensions/mcp "$ROOT/extensions/mcp"
fi
cp -R services/. "$ROOT/services/"
cp -R "$DIR" "$ROOT/services/profiles/$PROFILE"

cat > "$ROOT/UNPACK.md" <<EOF
# systemprompt self-host bundle — profile: $PROFILE

Unpack under /app (or any root; keep the layout) and on first run:

1. Verify the database in services/profiles/$PROFILE/secrets.json is reachable
   from this machine: pg_isready -h <host> -p <port>. On Oracle Cloud, open
   ingress for the Postgres port in the OCI security list AND the VM firewall.
2. Migrate with THIS binary before the first start:
   bin/systemprompt infra db migrate --profile $PROFILE
3. Serve (wrap in systemd or docker for restarts):
   SYSTEMPROMPT_PROFILE=\$PWD/services/profiles/$PROFILE/profile.yaml \\
   SYSTEMPROMPT_SERVICES_PATH=\$PWD/services \\
   SYSTEMPROMPT_TEMPLATES_PATH=\$PWD/services/web/templates \\
   SYSTEMPROMPT_ASSETS_PATH=\$PWD/services/web/assets \\
   PATH=\$PWD/bin:\$PATH \\
   bin/systemprompt infra services serve --foreground
4. Probe: curl -s localhost:<server.port>/api/v1/health
   Open ingress for the HTTP port before testing externally.

Validate the profile any time with: scripts/profile-check.sh $PROFILE
(from a repo clone) — or re-run step 1 and 4 here.
EOF

mkdir -p dist
OUT="dist/systemprompt-$PROFILE.tar.gz"
tar -czf "$OUT" -C "$STAGE" "systemprompt-$PROFILE"
echo "bundled: $OUT ($(du -h "$OUT" | cut -f1 | tr -d ' '))"
