#!/usr/bin/env python3
"""Verify authenticated read-only Atlassian access and retained fixture mutation evidence."""

import argparse
import json
from pathlib import Path
import urllib.error
import urllib.request


def request(url, token, body=None, headers=None):
    values = {"accept": "application/json,text/event-stream", **(headers or {})}
    if token:
        values["authorization"] = "Bearer " + token
    data = None
    if body is not None:
        values["content-type"] = "application/json"
        data = json.dumps(body, separators=(",", ":")).encode()
    call = urllib.request.Request(url, data=data, headers=values)
    try:
        with urllib.request.urlopen(call, timeout=90) as response:
            return decode(response.read()), dict(response.headers)
    except urllib.error.HTTPError as error:
        raise RuntimeError(f"HTTP {error.code}: {error.read().decode()[:1000]}") from None


def decode(raw):
    text = raw.decode()
    if not text.strip():
        return {}
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        events = []
        for line in text.splitlines():
            if line.startswith("data:"):
                try:
                    events.append(json.loads(line[5:].strip()))
                except json.JSONDecodeError:
                    pass
        if not events:
            raise RuntimeError("MCP response contained no JSON event")
        return events[-1]


def admin_get(base, access, path):
    value, _ = request(base + path, access)
    return value


def artifact_texts(base, access, experiment):
    detail = admin_get(base, access, f"/api/public/admin/evals/experiments/{experiment}")
    for execution in detail.get("executions", []):
        execution_id = execution["id"]
        try:
            archive = admin_get(base, access, f"/api/public/admin/evals/executions/{execution_id}/artifacts")
        except RuntimeError:
            continue
        for file in archive.get("files", {}).values():
            raw = file.get("bytes", [])
            if isinstance(raw, list):
                yield bytes(raw).decode("utf-8", "replace")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", default="http://localhost:8080")
    parser.add_argument("--pat-file", type=Path, required=True)
    parser.add_argument("--experiment", required=True)
    parser.add_argument("--evidence-dir", type=Path, required=True)
    args = parser.parse_args()
    args.evidence_dir.mkdir(parents=True, exist_ok=True)
    args.evidence_dir.chmod(0o700)

    pat = args.pat_file.read_text().strip()
    issued, _ = request(args.base + "/v1/auth/bridge/pat", pat, {})
    access = issued["token"]
    endpoint = args.base + "/api/v1/mcp/atlassian/mcp"
    initialized, response_headers = request(endpoint, access, {
        "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "systemprompt-evaluation-acceptance", "version": "1"},
        },
    })
    session = next((value for key, value in response_headers.items() if key.lower() == "mcp-session-id"), None)
    if not session or "result" not in initialized:
        raise RuntimeError("Authenticated Atlassian MCP initialization failed")
    session_header = {"mcp-session-id": session}
    request(endpoint, access, {"jsonrpc": "2.0", "method": "notifications/initialized"}, session_header)
    listed, _ = request(endpoint, access, {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}, session_header)
    tools = listed.get("result", {}).get("tools", [])
    selected = next((tool for tool in tools if tool.get("name") == "getAccessibleAtlassianResources"), None)
    if not selected or selected.get("annotations", {}).get("readOnlyHint") is not True:
        raise RuntimeError("The authenticated Atlassian resource tool is absent or not read-only")
    read_result, _ = request(endpoint, access, {
        "jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": {"name": selected["name"], "arguments": {}},
    }, session_header)
    if read_result.get("error") or read_result.get("result", {}).get("isError") is True:
        raise RuntimeError("Authenticated Atlassian resource read failed")

    retained = "\n".join(artifact_texts(args.base, access, args.experiment))
    required = [
        "fixture:platform_test_record",
        "fixture:platform_test_record_write",
        "fixture:platform_test_record_restore",
        '"readback_verified":true',
    ]
    missing = [label for label in required if label not in retained.replace(" ", "")]
    if missing:
        raise RuntimeError("Paid experiment lacks retained platform test-record evidence: " + ", ".join(missing))

    evidence = {
        "experiment_id": args.experiment,
        "atlassian_operation": selected["name"],
        "read_only_annotation": True,
        "authenticated_read_succeeded": True,
        "platform_test_record_read_write_readback_restore": True,
    }
    output = args.evidence_dir / "atlassian-acceptance.json"
    output.write_text(json.dumps(evidence, indent=2) + "\n")
    output.chmod(0o600)
    print(output)


if __name__ == "__main__":
    main()
