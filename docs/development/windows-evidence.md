# Windows evidence run

A runbook for an agent session on a Windows machine. It builds the harness
workspace and the Android app from the `future` branches, exercises the
phone-facing routes of a running instance, and writes what it saw to a folder
that is never committed. Follow it top to bottom. Where a step cannot run,
record it as skipped with the reason and carry on; do not improvise a
substitute.

Everything here is read from the tree at the SHA you clone. If a recipe or
file named below is missing at that SHA, the tree has moved: record that and
use what the README beside it says instead.

## 1. Prerequisites

Run in Git Bash (the scripts are bash). Versions are the pins in
`extensions/mobile/shared/toolchain.env`; `just mobile-toolchain` prints them
and fails on drift, so that file wins over this table.

| Tool | Version | Check |
|---|---|---|
| Git for Windows (Git Bash) | current | `git --version` |
| rustup | current; the toolchain comes from `rust-toolchain.toml` | `rustup show` inside the clone |
| just | current | `just --version` |
| Python | 3.10 or newer, as `python3` on PATH | `python3 --version` |
| JDK | 17 | `java -version` |
| Android SDK | platform 36, build tools 36.0.0 | `ANDROID_HOME` set |
| Android NDK | 29.0.14206865 | `just mobile-android-sdk --ndk` installs it |
| cargo-ndk | 4.1.2 | `cargo ndk --version` |
| Rust Android targets | `aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android` | `rustup target list --installed` |
| GitHub CLI | current, signed in | `gh auth status` |

Gradle (9.9.2) comes from the wrapper; do not install it. Symbolic links are
not needed: nothing in either repository relies on them.

An emulator needs hardware acceleration (Windows Hypervisor Platform). A
physical device with USB debugging works as well; say which you used.

## 2. Clone and record

```bash
mkdir sp && cd sp
git clone -b future https://github.com/systempromptio/systemprompt-core
git clone -b future https://github.com/systempromptio/systemprompt-template
mkdir -p systemprompt-template/evidence
git -C systemprompt-core rev-parse HEAD     > systemprompt-template/evidence/core.sha
git -C systemprompt-template rev-parse HEAD > systemprompt-template/evidence/template.sha
cat systemprompt-template/bridge/CORE_REF   > systemprompt-template/evidence/core-ref.txt
```

`evidence/` is the evidence folder. It must stay untracked: `.gitignore` lists it, so
`git status --short evidence` prints nothing. Never `git add -f` it.

`bridge/CORE_REF` is the core commit the template builds against. The
phone-facing server routes in section 5 exist from core `c1d24b20a` onward.
If `CORE_REF` is older (`git -C ../systemprompt-core merge-base --is-ancestor c1d24b20a "$(cat bridge/CORE_REF)"`
fails), the server you build from this template does not have them: run
sections 3 and 4, and record section 5 as skipped for that reason.

## 3. Harness

Read `extensions/harness/README.md` at your SHA first; it states what the
workspace holds and which recipes exist. Run exactly the recipes its
"Recipes" section lists, from the template root. At the time of writing
those are:

```bash
just harness-toolchain 2>&1 | tee evidence/harness-toolchain.log
just harness-resolve   2>&1 | tee evidence/harness-resolve.log
```

If the README lists build, check or test recipes, run each the same way with
its own log. Record the exit status of each.

## 4. Android

From the template root:

```bash
just mobile-toolchain        2>&1 | tee evidence/mobile-toolchain.log
just mobile-android-sdk --ndk 2>&1 | tee evidence/android-sdk.log
just mobile-rust-targets     2>&1 | tee evidence/rust-targets.log
just mobile-android-library  2>&1 | tee evidence/android-lib.log
just mobile-android          2>&1 | tee evidence/android-apk.log
just mobile-android-check    2>&1 | tee evidence/android-check.log
```

`mobile-android-library` builds the Rust library for the three ABIs and
generates the Kotlin bindings (both are build output); `mobile-android`
builds the debug APK. Record:

- the size of `libsystemprompt_mobile_app.so` per ABI under the app's `jniLibs`;
- the path, size and SHA-256 of the debug APK (`certutil -hashfile <apk> SHA256`).

### 4.1 Run it

```bash
just mobile-stub-instance &          # the stub discovery server, port 8765
just mobile-android-emulator         # or connect a device; `just mobile-android-emulator stop` ends it
just mobile-android-test 2>&1 | tee evidence/android-connected.log
```

The instrumented test types the stub's address into the app and expects the
discovered display name. From an emulator the host machine is `10.0.2.2`;
from a device use the machine's LAN address and allow the port through the
firewall.

Then install the debug APK (`just mobile-android-install`, or `adb install -r <apk>`), enter
the address by hand, and capture the screen after a successful discovery:

```bash
adb exec-out screencap -p > evidence/android-discovery.png
```

### 4.2 Discovery against a real server

Use an instance reachable from the Windows machine: one you start from this
template (`just setup-local`, `just start`; set `server.host` to
`127.0.0.1` or the LAN address, not `localhost`) or one you are given. Plain
HTTP is accepted by the app's debug build only, and only for loopback and
private addresses.

```bash
BASE=http://<host>:<port>
curl -sS -D evidence/discovery.headers "$BASE/.well-known/systemprompt-bridge" -o evidence/discovery.json
```

Enter the same address in the app and screenshot the result to
`evidence/android-discovery-real.png`.

## 5. Phone-facing routes

These need a server built on core `c1d24b20a` or later (section 2). The
Android parts also need the instance to declare an Android client: in the
gateway config, `bridge_instance.clients.android` (an `app_id`) together
with an `android` entry in `bridge_releases.assets`. A config with the
first and not the second is refused at load. If the instance declares
neither, the three Android requests below answer 404 or 401; record the
status and move on.

### 5.1 Association file

```bash
curl -sS -D evidence/assetlinks.headers "$BASE/.well-known/assetlinks.json" -o evidence/assetlinks.json
```

Expected when `bridge_instance.vouched_apps.android` is set: 200, JSON, a
cache header, one entry per vouched package with its certificate
fingerprints. No credential is sent.

### 5.2 The APK route and its checksum

```bash
curl -sS "$BASE/v1/bridge/latest?platform=android" -o evidence/android-latest.json
curl -sS -D evidence/android-download.headers "$BASE/v1/bridge/download/android" -o evidence/android.apk
certutil -hashfile evidence/android.apk SHA256 > evidence/android.apk.sha256
```

No credential is sent. `android-latest.json` carries `version`, `sha256`
and `size`; the hash and byte count of the downloaded file must equal them.
Record both values and whether they match. Then confirm another platform is
still closed: `curl -s -o /dev/null -w '%{http_code}' "$BASE/v1/bridge/latest?platform=windows-x64"`
must not be 200 without a credential.

### 5.3 Register and revoke a push token

This needs a signed-in user's bearer token and the fingerprint of a device
that user has enrolled (the value the device was enrolled under through the
existing device-link exchange). If you have no enrolled device, record the
403 the first request returns (`device_not_enrolled`) as the result: that is
the route refusing an unknown device, which is also worth having on record.

```bash
curl -sS -X PUT "$BASE/api/v1/notifications/push-token" \
  -H "authorization: Bearer $TOKEN" \
  -H "x-systemprompt-device-fingerprint: $FINGERPRINT" \
  -H 'content-type: application/json' \
  -d '{"provider":"fcm","token":"evidence-run-token-0001","client":{"platform":"android","os_version":"16","app_version":"0.70.0","app_id":"io.systemprompt.mobile"}}' \
  -o evidence/push-register.json -w '%{http_code}\n'

curl -sS -X DELETE "$BASE/api/v1/notifications/push-token" \
  -H "authorization: Bearer $TOKEN" \
  -H "x-systemprompt-device-fingerprint: $FINGERPRINT" \
  -o evidence/push-revoke.json -w '%{http_code}\n'
```

Expected: the first reply names a `device_id`, the provider and
`registered_at` and does not echo the token; the second reports
`"revoked": true`; repeating the second reports `"revoked": false`. Never
write `$TOKEN` into the evidence folder or the report.

## 6. Report

Write `evidence/REPORT.md` in this shape and paste it back to whoever asked
for the run. One line per step; no prose beyond the notes column.

```text
Windows evidence run, <date>, <Windows version>, <emulator image or device model>
core <sha>   template <sha>   CORE_REF <sha>

| Section | Step | Result (pass / fail / skipped) | Notes: exit code, sizes, hashes, status codes, the reason for a skip |
| 3 | harness-toolchain | | |
| 3 | harness-resolve | | |
| 4 | toolchain pins | | |
| 4 | library, three ABIs | | sizes |
| 4 | debug APK | | path, size, sha256 |
| 4 | lint and unit tests | | |
| 4.1 | connected tests against the stub | | test count |
| 4.1 | screenshot | | file name |
| 4.2 | discovery against a server | | address kind (loopback, LAN, public) |
| 5.1 | assetlinks.json | | status |
| 5.2 | latest + download + checksum | | match or mismatch |
| 5.3 | push token register / revoke | | status codes |

Failures: for each, the command, the last 30 lines of its log, and what you
checked before concluding it is not the machine.
```

Do not commit, push, open an issue or post anything from this run. The
report and the folder are the whole output.
