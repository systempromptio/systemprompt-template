#!/usr/bin/env python3
"""A stub instance for development and the Android instrumented test.

It answers one request, the discovery document an app reads before anyone has
signed in (GET /.well-known/systemprompt-bridge), and nothing else. It is not
an instance: there is no sign-in and no data behind it.

    stub-instance.py [--port 8765] [--name "Stub Instance"]   serve until interrupted
    stub-instance.py --probe [--port 8765]                    exit 0 when one is already listening

It listens on the loopback address only. An Android emulator reaches the
host's loopback at 10.0.2.2, so the address to type in the app is
http://10.0.2.2:8765; the core admits plain HTTP for loopback and private
hosts only, and the app permits it in its debug build only.
"""

import argparse
import json
import sys
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

DISCOVERY_PATH = "/.well-known/systemprompt-bridge"
DEFAULT_PORT = 8765
DEFAULT_NAME = "Stub Instance"
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


def probe(port):
    try:
        with urllib.request.urlopen(f"http://127.0.0.1:{port}{DISCOVERY_PATH}", timeout=2) as response:
            return json.load(response).get("contract") == "1"
    except (urllib.error.URLError, OSError, ValueError):
        return False


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--port", type=int, default=DEFAULT_PORT)
    parser.add_argument("--name", default=DEFAULT_NAME)
    parser.add_argument("--probe", action="store_true")
    args = parser.parse_args()
    if args.probe:
        return 0 if probe(args.port) else 1
    server = ThreadingHTTPServer(("127.0.0.1", args.port), handler(args.name))
    print(f"stub instance '{args.name}' on http://127.0.0.1:{args.port} (emulator: http://10.0.2.2:{args.port})", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
