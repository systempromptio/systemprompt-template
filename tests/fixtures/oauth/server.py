"""Loopback TLS OAuth/MCP fixture; no external accounts or network services."""
import base64
import http.server
import json
import pathlib
import ssl
import subprocess
import sys
import urllib.parse

root = pathlib.Path(sys.argv[1])
def openssl(*args):
    subprocess.run(["openssl", *args], cwd=root, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

openssl("req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", "ca.key", "-out", "ca.pem", "-days", "1", "-subj", "/CN=OAuth test CA")
openssl("req", "-newkey", "rsa:2048", "-nodes", "-keyout", "server.key", "-out", "server.csr", "-subj", "/CN=localhost")
(root / "extensions").write_text("subjectAltName=DNS:localhost,IP:127.0.0.1\nbasicConstraints=CA:FALSE\nkeyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n")
openssl("x509", "-req", "-in", "server.csr", "-CA", "ca.pem", "-CAkey", "ca.key", "-CAcreateserial", "-out", "server.pem", "-days", "1", "-extfile", "extensions")

class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def send_json(self, obj, status=200):
        data = json.dumps(obj).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        if self.path.startswith("/.well-known/oauth-protected-resource"):
            suffix = self.path.removeprefix("/.well-known/oauth-protected-resource") or "/mcp"
            # Google advertises the issuer with a trailing slash here and without
            # one in its OpenID metadata; the client must treat them as one issuer.
            return self.send_json({"resource": origin + suffix, "authorization_servers": [origin + "/"]})
        if self.path == "/.well-known/oauth-authorization-server":
            return self.send_json({"issuer": origin, "authorization_endpoint": origin + "/authorize", "token_endpoint": origin + "/token",
                "registration_endpoint": origin + "/register", "userinfo_endpoint": origin + "/userinfo",
                "code_challenge_methods_supported": ["S256"],
                "token_endpoint_auth_methods_supported": ["none", "client_secret_post", "client_secret_basic"]})
        if self.path == "/userinfo":
            if self.headers.get("Authorization") != "Bearer access-token":
                return self.send_json({}, 401)
            return self.send_json({"sub": "1029384756", "email": "pilot.tester@example.test", "name": "Pilot Tester"})
        self.send_json({}, 404)

    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("Content-Length", 0))).decode()
        if self.path == "/register":
            registration = json.loads(body)
            if registration.get("token_endpoint_auth_method") != "none":
                return self.send_json({}, 400)
            return self.send_json({"client_id": "test-client"})
        if self.path == "/token":
            fields = urllib.parse.parse_qs(body)
            if fields.get("grant_type") == ["authorization_code"]:
                valid = fields.get("code") == ["test-code"] and fields.get("code_verifier") == ["test-verifier"]
            else:
                valid = fields.get("refresh_token") == ["refresh-token"]
            if not valid or fields.get("client_id") != ["test-client"] or fields.get("resource") != [origin + "/mcp"]:
                return self.send_json({"error": "invalid_grant"}, 400)
            return self.send_json({"access_token": "access-token", "token_type": "Bearer", "refresh_token": "refresh-token", "expires_in": 3600})
        if self.path == "/mcp":
            if self.headers.get("Authorization") != "Bearer access-token":
                return self.send_json({}, 401)
            message = json.loads(body)
            result = {"protocolVersion": "2025-03-26", "capabilities": {"tools": {}}, "serverInfo": {"name": "fourth-mcp", "version": "1"}}
            if message["method"] == "tools/list":
                result = {"tools": [{"name": "read_example", "inputSchema": {"type": "object"}}]}
            if "id" not in message:
                return self.send_json({})
            return self.send_json({"jsonrpc": "2.0", "id": message["id"], "result": result})
        self.send_json({}, 404)

    def do_DELETE(self):
        self.send_json({})

server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
origin = "https://localhost:" + str(server.server_port)
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(root / "server.pem", root / "server.key")
server.socket = context.wrap_socket(server.socket, server_side=True)
print(origin, flush=True)
server.serve_forever()
