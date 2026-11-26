#!/usr/bin/env python3
"""Sign in to the console the way a browser does, with a software passkey.

    console-login.py BASE ORIGIN EMAIL SETUP_TOKEN

Links a fresh P-256 passkey to EMAIL with an `admin users webauthn
generate-setup-token` token, asserts it through /auth/start and /auth/finish,
completes the PKCE authorization the login page runs, stores the session
through /api/public/auth/session and prints the `access_token=…` cookie.
BASE is where the server answers; ORIGIN is its configured external URL,
which the WebAuthn client data must name."""
import base64, hashlib, json, os, secrets, sys, urllib.parse, urllib.request, http.client
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.hazmat.primitives import hashes

base, origin, email, setup_token = sys.argv[1:5]

def b64u(b): return base64.urlsafe_b64encode(b).rstrip(b"=").decode()
def unb64u(s): return base64.urlsafe_b64decode(s + "=" * (-len(s) % 4))

def cbor(v):
    def head(major, n):
        if n < 24: return bytes([major << 5 | n])
        if n < 256: return bytes([major << 5 | 24, n])
        if n < 65536: return bytes([major << 5 | 25]) + n.to_bytes(2, "big")
        return bytes([major << 5 | 26]) + n.to_bytes(4, "big")
    if isinstance(v, bool): return b"\xf5" if v else b"\xf4"
    if isinstance(v, int): return head(0, v) if v >= 0 else head(1, -1 - v)
    if isinstance(v, bytes): return head(2, len(v)) + v
    if isinstance(v, str): e = v.encode(); return head(3, len(e)) + e
    if isinstance(v, dict): return head(5, len(v)) + b"".join(cbor(k) + cbor(x) for k, x in v.items())
    raise TypeError(v)

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *a, **k): return None
opener = urllib.request.build_opener(NoRedirect)

def req(method, path, body=None, form=False, headers=None):
    h = {"Origin": origin, **(headers or {})}
    data = None
    if body is not None:
        if form: data = urllib.parse.urlencode(body).encode(); h["Content-Type"] = "application/x-www-form-urlencoded"
        else: data = json.dumps(body).encode(); h["Content-Type"] = "application/json"
    r = urllib.request.Request(base + path, data=data, method=method, headers=h)
    try: resp = opener.open(r)
    except urllib.error.HTTPError as e: resp = e
    raw = resp.read()
    if resp.status >= 400: sys.exit(f"{method} {path} -> {resp.status}: {raw[:400]!r}")
    return resp, (json.loads(raw) if raw and raw[:1] in b"{[" else raw)

key = ec.generate_private_key(ec.SECP256R1())
nums = key.public_key().public_numbers()
cred_id = secrets.token_bytes(32)
W = "/api/v1/core/oauth/webauthn"

resp, start = req("GET", f"{W}/link/start?token={urllib.parse.quote(setup_token)}")
link_challenge_id = resp.headers["x-challenge-id"]
pk = start["challenge"]["publicKey"]
rp_hash = hashlib.sha256(pk["rp"]["id"].encode()).digest()
user_handle = unb64u(pk["user"]["id"])
cose = cbor({1: 2, 3: -7, -1: 1, -2: nums.x.to_bytes(32, "big"), -3: nums.y.to_bytes(32, "big")})
auth_data = rp_hash + bytes([0x45]) + (0).to_bytes(4, "big") + bytes(16) + len(cred_id).to_bytes(2, "big") + cred_id + cose
cdj = json.dumps({"type": "webauthn.create", "challenge": pk["challenge"], "origin": origin, "crossOrigin": False}).encode()
att = cbor({"fmt": "none", "attStmt": {}, "authData": auth_data})
req("POST", f"{W}/link/finish", {"challenge_id": link_challenge_id, "token": setup_token,
    "credential": {"id": b64u(cred_id), "rawId": b64u(cred_id), "type": "public-key",
                   "response": {"attestationObject": b64u(att), "clientDataJSON": b64u(cdj)}}})
print("linked passkey", file=sys.stderr)

_, s = req("POST", f"{W}/auth/start?email={urllib.parse.quote(email)}")
apk = s["publicKey"]
ad = hashlib.sha256(apk["rpId"].encode()).digest() + bytes([0x05]) + (1).to_bytes(4, "big")
cdj = json.dumps({"type": "webauthn.get", "challenge": apk["challenge"], "origin": origin, "crossOrigin": False}).encode()
sig = key.sign(ad + hashlib.sha256(cdj).digest(), ec.ECDSA(hashes.SHA256()))
_, fin = req("POST", f"{W}/auth/finish", {"challenge_id": s["challenge_id"],
    "credential": {"id": b64u(cred_id), "rawId": b64u(cred_id), "type": "public-key",
                   "response": {"authenticatorData": b64u(ad), "clientDataJSON": b64u(cdj), "signature": b64u(sig), "userHandle": b64u(user_handle)}}})
print("passkey assertion accepted", file=sys.stderr)

verifier = secrets.token_urlsafe(48)
challenge = b64u(hashlib.sha256(verifier.encode()).digest())
state = secrets.token_urlsafe(24)
q = urllib.parse.urlencode({"user_id": fin["user_id"], "auth_token": fin["auth_token"], "response_type": "code",
    "client_id": "marketplace-admin", "redirect_uri": origin + "/admin/login", "scope": "user", "state": state,
    "code_challenge": challenge, "code_challenge_method": "S256"})
resp, body = req("GET", f"{W}/complete?{q}", headers={"Accept": "application/json"})
params = {"code": body["authorization_code"], "state": body["state"]} if isinstance(body, dict) else dict(urllib.parse.parse_qsl(urllib.parse.urlparse(resp.headers["Location"]).query))
assert params.get("state") == state, params
print("authorization code issued", file=sys.stderr)
_, tok = req("POST", "/api/v1/core/oauth/token", {"grant_type": "authorization_code", "code": params["code"],
    "redirect_uri": origin + "/admin/login", "code_verifier": verifier, "client_id": "marketplace-admin"}, form=True)
resp, _ = req("POST", "/api/public/auth/session", {"access_token": tok["access_token"], "expires_in": tok.get("expires_in", 3600)})
cookie = [c for c in resp.headers.get_all("Set-Cookie") if c.startswith("access_token=")][0].split(";")[0]
print("session stored", file=sys.stderr)
print(cookie)
