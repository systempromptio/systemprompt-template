#!/usr/bin/env python3
"""A stub instance for development and the Android instrumented test.

It answers one request, the discovery document an app reads before anyone has
signed in (GET /.well-known/systemprompt-bridge), and nothing else. It is not
an instance: there is no sign-in and no data behind it.

    stub-instance.py [--port 8765] [--name "Stub Instance"]   serve until interrupted
    stub-instance.py --probe [--port 8765]                    exit 0 when one is already listening
    stub-instance.py --releases APK [--port 8766] [--repo OWNER/NAME] [--version X.Y.Z]

--releases is a second, separate stand-in: the release API a real instance
reads its app download from. It serves one release, tagged bridge-vX.Y.Z, whose
assets are the file APK and a SHA256SUMS naming it, in the three requests the
gateway makes: the release list, the checksum file and the package. A local
instance whose gateway.bridge_releases.api_base is this address (with
repo OWNER/NAME and assets.android the APK's file name) then answers
/v1/bridge/latest and /v1/bridge/download/android with this file.

It listens on the loopback address only. An Android emulator reaches the
host's loopback at 10.0.2.2, so the address to type in the app is
http://10.0.2.2:8765; the core admits plain HTTP for loopback and private
hosts only, and the app permits it in its debug build only.
"""

import argparse
import hashlib
import json
import os
import sys
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

DISCOVERY_PATH = "/.well-known/systemprompt-bridge"
DEFAULT_PORT = 8765
DEFAULT_NAME = "Stub Instance"
DEFAULT_RELEASES_PORT = 8766
DEFAULT_REPO = "example/mobile-releases"
DEFAULT_VERSION = "0.1.0"
SUMS = "SHA256SUMS"
APP_ID = "io.systemprompt.mobile"


def document(name):
    return {
        "contract": "1",
        "device_link_path": "/api/v1/bridge/device-link",
        "identity_path": "/api/v1/bridge/identity",
        "branding": {"display_name": name, "sign_in_hint": "Use your work account"},
        "enabled_hosts": [],
        "clients": {
            "ios": {"app_id": APP_ID, "link": "https://testflight.example/join/stub"},
            "android": {
                "app_id": APP_ID,
                "min_version": "0.70.0",
                "download_path": "/v1/bridge/download/android",
            },
        },
    }


def handler(name):
    body = json.dumps(document(name)).encode()

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path != DISCOVERY_PATH:
                self.send_error(404)
                return
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    return Handler


def releases_handler(apk, repo, version, port):
    name = os.path.basename(apk)
    with open(apk, "rb") as package:
        payload = package.read()
    sums = f"{hashlib.sha256(payload).hexdigest()}  {name}\n".encode()
    base = f"http://127.0.0.1:{port}"
    release = {
        "tag_name": f"bridge-v{version}",
        "draft": False,
        "prerelease": False,
        "html_url": f"{base}/release",
        "assets": [
            {"name": name, "url": f"{base}/asset/{name}", "size": len(payload)},
            {"name": SUMS, "url": f"{base}/asset/{SUMS}", "size": len(sums)},
        ],
    }
    listing = json.dumps([release]).encode()
    answers = {
        f"/repos/{repo}/releases": ("application/json", listing),
        f"/repos/{repo}/releases/tags/bridge-v{version}": ("application/json", json.dumps(release).encode()),
        f"/asset/{name}": ("application/octet-stream", payload),
        f"/asset/{SUMS}": ("application/octet-stream", sums),
    }

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            answer = answers.get(self.path.split("?", 1)[0])
            if answer is None:
                self.send_error(404)
                return
            media_type, body = answer
            self.send_response(200)
            self.send_header("Content-Type", media_type)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    return Handler


def probe(port):
    try:
        with urllib.request.urlopen(f"http://127.0.0.1:{port}{DISCOVERY_PATH}", timeout=2) as response:
            return json.load(response).get("contract") == "1"
    except (urllib.error.URLError, OSError, ValueError):
        return False


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--port", type=int)
    parser.add_argument("--name", default=DEFAULT_NAME)
    parser.add_argument("--probe", action="store_true")
    parser.add_argument("--releases", metavar="APK")
    parser.add_argument("--repo", default=DEFAULT_REPO)
    parser.add_argument("--version", default=DEFAULT_VERSION)
    args = parser.parse_args()
    if args.releases:
        port = args.port or DEFAULT_RELEASES_PORT
        server = ThreadingHTTPServer(("127.0.0.1", port), releases_handler(args.releases, args.repo, args.version, port))
        print(f"stub release API for {args.repo} (bridge-v{args.version}, {os.path.basename(args.releases)}) on http://127.0.0.1:{port}", flush=True)
    else:
        port = args.port or DEFAULT_PORT
        if args.probe:
            return 0 if probe(port) else 1
        server = ThreadingHTTPServer(("127.0.0.1", port), handler(args.name))
        print(f"stub instance '{args.name}' on http://127.0.0.1:{port} (emulator: http://10.0.2.2:{port})", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
