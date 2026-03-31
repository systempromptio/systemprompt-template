# Demo 06: Secret Injection Breach — Architecture

## What it does

Tests the secret detection rule with 4 direct API calls: AWS key, GitHub PAT, private key (all DENIED), and clean input (ALLOWED).

## Flow (per test)

```
  curl POST /api/public/hooks/govern
  Body: { tool_input: { command: "...AKIAIOSFODNN7EXAMPLE..." } }
    │
    ▼
  ┌─────────────────────────────────────────────────────────┐
  │  JWT Validation → UserId                                │
  │  Scope Resolution → "admin" (developer_agent)           │
  │  Note: admin scope does NOT bypass secret detection     │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Rule Engine                                            │
  │                                                         │
  │  Rule 1: scope_check → PASS (admin scope)              │
  │                                                         │
  │  Rule 2: secret_injection                               │
  │    Scans ALL string values in tool_input recursively    │
  │    Pattern matching:                                    │
  │      AKIA[0-9A-Z]{16}  → AWS Access Key                │
  │      ghp_[a-zA-Z0-9]{36} → GitHub PAT                  │
  │      -----BEGIN.*PRIVATE KEY----- → PEM Key             │
  │    → FAIL (secret detected)                             │
  │                                                         │
  │  decision = "deny"                                      │
  │  policy = "secret_injection"                            │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Audit: INSERT governance_decisions                     │
  │  decision="deny", policy="secret_injection"             │
  │  reason: type of secret detected                        │
  │  Note: the secret value itself is NOT stored            │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  Response: { permissionDecision: "deny",
              permissionDecisionReason: "[GOVERNANCE] ..." }
```

## Test Matrix

| Test | Tool | Input Contains | Detection Pattern | Result |
|------|------|---------------|-------------------|--------|
| 1 | Bash | `AKIAIOSFODNN7EXAMPLE` | AWS key regex | DENY |
| 2 | Write | `ghp_ABCDEFghijklmnop...` | GitHub PAT regex | DENY |
| 3 | Write | `-----BEGIN RSA PRIVATE KEY-----` | PEM header match | DENY |
| 4 | Read | `/home/user/project/src/main.rs` | No secrets | ALLOW |

## Why Rust

- **Recursive scanning**: The secret scanner traverses `serde_json::Value` recursively — all nested strings in the tool_input are checked, not just top-level fields
- **Pattern safety**: Regex patterns are compiled once at startup and reused — no per-request compilation overhead
- **Audit without leaking**: The audit record stores the secret TYPE (e.g., "AWS access key") but NOT the actual secret value — typed structs enforce this separation
- **Defense in depth**: Secret detection is independent of scope — even `admin` agents are blocked. This is enforced by the rule engine's typed evaluation pipeline, not by conditional string checks
