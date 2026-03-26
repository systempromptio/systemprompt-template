# Odoo API Protocol Guide

## Overview

This skill supports two protocols for connecting to Odoo:

## Protocol Selection Rules

**CRITICAL - Protocol selection is strictly version-based:**
- **Odoo >= 19.0**: MUST use JSON2 protocol
- **Odoo <= 18.0**: MUST use JSON-RPC protocol

**There is no choice** - the protocol is determined by the Odoo version.

### JSON-RPC (Odoo <= 18.0)
- **Versions**: Odoo 8.0 to 18.0 (inclusive)
- **Authentication**: Username/Password or API Key
- **Endpoints**: `/jsonrpc`, `/web/session/authenticate`
- **Session**: Cookie-based (stateful)
- **Status**: Will be removed in Odoo 20.0, but still the only option for Odoo <= 18.0

### JSON2 (Odoo >= 19.0)
- **Versions**: Odoo 19.0 and later
- **Authentication**: API Key via Bearer token
- **Endpoint**: `/json/2/{model}/{method}` (NOT `/json/2/call`)
- **Session**: Stateless (Bearer token in each request)
- **Status**: Official protocol for Odoo 19.0+, not available in earlier versions

## Script Organization

All scripts in the `scripts/` directory support **both protocols** automatically based on version:

```
scripts/
├── auth.sh              # Unified auth (auto-detects version, RECOMMENDED)
├── auth_jsonrpc.sh      # JSON-RPC auth (Odoo <= 18.0 only)
├── auth_json2.sh        # JSON2 auth (Odoo >= 19.0 only)
├── detect_version.sh    # Auto-detect Odoo version
├── search_records.sh    # Search records (version-aware)
├── create_record.sh     # Create records (version-aware)
├── update_record.sh     # Update records (version-aware)
├── delete_record.sh     # Delete records (version-aware)
├── install_module.sh    # Install modules (version-aware)
├── uninstall_module.sh  # Uninstall modules (version-aware)
├── upgrade_module.sh    # Upgrade modules (version-aware)
└── execute_method.sh    # Execute any method (version-aware)
```

## Protocol Detection

Scripts automatically select the correct protocol based on Odoo version via the `ODOO_PROTOCOL` environment variable set by authentication:

```bash
# Odoo >= 19.0 → Sets ODOO_PROTOCOL="json2"
# Odoo <= 18.0 → Sets ODOO_PROTOCOL="jsonrpc"

# Best practice: Use auth.sh for automatic detection
eval $(./scripts/auth.sh)
```

**Version-Protocol Mapping:**
- Odoo 8.x → JSON-RPC
- Odoo 9.x → JSON-RPC
- Odoo 10.x → JSON-RPC
- Odoo 11.x → JSON-RPC
- Odoo 12.x → JSON-RPC
- Odoo 13.x → JSON-RPC
- Odoo 14.x → JSON-RPC
- Odoo 15.x → JSON-RPC
- Odoo 16.x → JSON-RPC
- Odoo 17.x → JSON-RPC
- Odoo 18.x → JSON-RPC
- **Odoo 19.x → JSON2** (breaking change)
- Odoo 20.x+ → JSON2 only (JSON-RPC removed)

If not set, scripts default to `jsonrpc` for backwards compatibility.

## Quick Start

### Using JSON-RPC (Odoo 8-18)

```bash
# Set credentials
export ODOO_URL="https://your-instance.odoo.com"
export ODOO_DB="your_database"
export ODOO_USER="user@example.com"
export ODOO_KEY="api-key-or-password"

# Authenticate (creates session)
eval $(./scripts/auth_jsonrpc.sh)

# Use any script (automatically uses JSON-RPC)
./scripts/search_records.sh res.partner '[]' '["name","email"]' 10
```

### Using JSON2 (Odoo 19+)

```bash
# Set credentials (API Key required)
export ODOO_URL="https://your-instance.odoo.com"
export ODOO_DB="your_database"
export ODOO_KEY="your-api-key-here"  # Must be API Key from Settings > Users > API Keys

# Authenticate (verifies API key, sets protocol)
eval $(./scripts/auth_json2.sh)

# Use any script (automatically uses JSON2 with Bearer token)
./scripts/search_records.sh res.partner '[]' '["name","email"]' 10
```

## Environment Variables

### Common Variables
- `ODOO_URL`: Your Odoo instance URL (e.g., https://demo.odoo.com)
- `ODOO_DB`: Database name
- `ODOO_KEY`: API Key or password

### JSON-RPC Specific
- `ODOO_USER`: User email/login
- `ODOO_SESSION_ID`: Session cookie (set by auth_jsonrpc.sh)
- `ODOO_UID`: User ID (set by auth_jsonrpc.sh)

### JSON2 Specific
- `ODOO_AUTH_HEADER`: Full Authorization header (set by auth_json2.sh)
- `ODOO_UID`: User ID (set by auth_json2.sh)

### Protocol Control
- `ODOO_PROTOCOL`: Either "jsonrpc" or "json2" (set by auth scripts)
- `ODOO_VERSION`: Odoo version string (set by auth scripts)

## API Key Generation

### For Odoo 19+ (JSON2)

1. Log in to your Odoo instance
2. Go to: **Preferences → Account Security → API Keys**
3. Click **New API Key**
4. Set a description (e.g., "Integration Script")
5. Set duration (recommended: short duration for security)
6. Copy the generated API key
7. Use it in the `ODOO_KEY` environment variable

**Important**:
- API keys can have a maximum duration of 3 months
- Keys cannot be retrieved after creation, store them securely
- For production, use short-lived keys (1 day recommended)

### For Odoo 8-18 (JSON-RPC)

API keys work the same way, but you can also use your account password in the `ODOO_KEY` variable.

## Protocol Comparison

| Feature | JSON-RPC | JSON2 |
|---------|----------|-------|
| Odoo Version | 8.0 - 18.0 | 19.0+ |
| Authentication | Username/Password or API Key | API Key only (Bearer) |
| Session Management | Cookie-based | Stateless |
| Endpoint | `/jsonrpc` | `/json/2/call` |
| Future Support | Deprecated (removed in v20) | Official protocol |
| Performance | Requires session | Better for stateless apps |

## Migration from JSON-RPC to JSON2

If you're upgrading from Odoo 18 to 19+:

1. Generate API keys for your users
2. Update authentication method:
   ```bash
   # Old (JSON-RPC)
   eval $(./scripts/auth_jsonrpc.sh)

   # New (JSON2)
   eval $(./scripts/auth_json2.sh)
   ```
3. Scripts will automatically use the correct protocol
4. No changes needed to other script calls

## Troubleshooting

### JSON2 Endpoint Not Found (404)

If `/json/2/call` returns 404:
- Your Odoo 19 instance may not have JSON2 enabled
- Check if it's a custom/modified installation
- Fallback to JSON-RPC (still works in v19)

### Authentication Failed with Bearer Token

Common causes:
- API key expired (check duration in Settings)
- API key not created via Settings > Users > API Keys
- Using password instead of API key (JSON2 requires API keys)
- API key permissions insufficient

### Session Cookie Issues (JSON-RPC)

If authentication works but operations fail:
- Re-run `auth_jsonrpc.sh` to refresh session
- Check if `ODOO_SESSION_ID` is set: `echo $ODOO_SESSION_ID`
- Session may have expired (default: 7 days)

## Security Best Practices

1. **Use API Keys**: Never use account passwords in production
2. **Short Duration**: Set API key duration to 1 day for interactive use
3. **Secure Storage**: Store API keys in environment variables or secret managers
4. **HTTPS Only**: Always use HTTPS in production
5. **Least Privilege**: Create dedicated integration users with minimal permissions
6. **Rotate Keys**: Regularly rotate API keys (before expiration)
7. **Never Commit**: Never commit credentials to version control

## Examples

### Complete Workflow (JSON2)

```bash
# 1. Setup
export ODOO_URL="https://your-instance.odoo.com"
export ODOO_DB="production"
export ODOO_KEY="your-api-key-here"

# 2. Authenticate
eval $(./scripts/auth_json2.sh)

# 3. Search for companies
./scripts/search_records.sh res.partner '[["is_company","=",true]]' '["name","email"]' 5

# 4. Create a new partner
./scripts/create_record.sh res.partner '{"name":"New Company","email":"info@company.com"}'

# 5. Install a module
./scripts/install_module.sh sale_management

# 6. Execute custom method
./scripts/execute_method.sh res.partner name_get '[[1,2,3]]' '{}'
```

### Switching Between Protocols

```bash
# Start with JSON-RPC
export ODOO_PROTOCOL="jsonrpc"
eval $(./scripts/auth_jsonrpc.sh)
./scripts/search_records.sh res.partner '[]'

# Switch to JSON2
export ODOO_PROTOCOL="json2"
eval $(./scripts/auth_json2.sh)
./scripts/search_records.sh res.partner '[]'
```

## Resources

- [Odoo 19.0 JSON2 API Documentation](https://www.odoo.com/documentation/19.0/developer/reference/external_api.html)
- [Odoo External API Guide](https://www.odoo.com/documentation/19.0/developer/howtos/web_services.html)
- [API Keys Setup](https://www.odoo.com/documentation/19.0/applications/general/users/api.html)
