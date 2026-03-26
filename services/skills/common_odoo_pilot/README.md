# Odoo Pilot - Quick Reference

Control and manage Odoo instances through API. Supports both Odoo < 19.0 (JSON-RPC) and Odoo >= 19.0 (JSON2) protocols.

## Structure

```
odoo-pilot/
├── SKILL.md                    # Main skill documentation (START HERE)
├── README.md                   # This file - quick reference
├── STRUCTURE.md                # File organization guide
├── PROTOCOL_GUIDE.md           # Protocol comparison & migration guide
├── JSON2_ENDPOINT_FORMAT.md    # JSON2 format reference (CRITICAL for JSON2)
├── EXAMPLES_JSON2.md           # Practical examples
├── CHANGELOG_JSON2.md          # Implementation details & fixes
└── scripts/
    ├── auth.sh                 # Unified authentication (RECOMMENDED)
    ├── auth_json2.sh           # JSON2 authentication (legacy)
    ├── auth_jsonrpc.sh         # JSON-RPC authentication (legacy)
    ├── detect_version.sh       # Auto-detect Odoo version
    ├── search_records.sh       # Search/read records
    ├── create_record.sh        # Create records
    ├── update_record.sh        # Update records
    ├── delete_record.sh        # Delete records
    ├── execute_method.sh       # Execute custom methods
    ├── install_module.sh       # Install modules
    ├── uninstall_module.sh     # Uninstall modules
    └── upgrade_module.sh       # Upgrade modules
```

## Documentation Guide

### When to Read Each File

1. **SKILL.md** - START HERE
   - Quick start guide
   - Script usage examples
   - Common workflows
   - Best practices

2. **JSON2_ENDPOINT_FORMAT.md** - READ THIS for JSON2 implementation
   - CRITICAL: Correct endpoint format (`/json/2/{model}/{method}`)
   - Request/response examples
   - cURL command examples
   - Comparison with JSON-RPC

3. **PROTOCOL_GUIDE.md** - Protocol selection & migration
   - When to use JSON2 vs JSON-RPC
   - Environment variable setup
   - Migration from JSON-RPC to JSON2
   - Troubleshooting

4. **EXAMPLES_JSON2.md** - Practical examples
   - CRUD operation examples
   - Module management workflows
   - Bulk operations
   - Domain syntax guide

5. **CHANGELOG_JSON2.md** - Technical details
   - Implementation history
   - Bug fixes and corrections
   - Technical explanations

## Quick Start

```bash
# 1. Set credentials (ALWAYS request from user)
export ODOO_URL="https://your-instance.odoo.com"
export ODOO_DB="your_database"
export ODOO_KEY="your_api_key"
export ODOO_USER="your_username"  # Only needed for JSON-RPC (Odoo < 19)

# 2. Authenticate (auto-detects protocol)
eval $(./scripts/auth.sh)

# 3. Use scripts
./scripts/search_records.sh res.partner '[]' '["name","email"]' 10
./scripts/install_module.sh sale_management
```

## Critical Information

### Protocol Selection by Version

**IMPORTANT - Version-based protocol rules:**
- **Odoo >= 19.0**: ALWAYS use JSON2 protocol
- **Odoo <= 18.0**: ALWAYS use JSON-RPC protocol

The `auth.sh` script automatically detects the version and uses the correct protocol.

### JSON2 Format (Odoo >= 19.0 ONLY)
**IMPORTANT**: JSON2 uses a different format than JSON-RPC:

- **Endpoint**: `/json/2/{model}/{method}` (NOT `/json/2/call`)
- **Headers**:
  - `Authorization: bearer {API_KEY}`
  - `X-Odoo-Database: {DB_NAME}`
  - `Content-Type: application/json`
- **Body**: Direct parameters (NO jsonrpc wrapper)

Example:
```bash
POST /json/2/res.partner/search_read HTTP/1.1
Authorization: bearer YOUR_API_KEY
X-Odoo-Database: your_database
Content-Type: application/json

{
  "domain": [["is_company", "=", true]],
  "fields": ["name", "email"],
  "limit": 10
}
```

**See JSON2_ENDPOINT_FORMAT.md for complete format reference.**

### JSON-RPC Format (Odoo <= 18.0 ONLY)
- **Endpoint**: `/jsonrpc`
- **Authentication**: Session-based (cookie)
- **Body**: JSON-RPC wrapper with execute_kw

## Security

- **NEVER hardcode credentials** in scripts or documentation
- Always request credentials from user at runtime
- Use API keys instead of passwords (Odoo >= 13.0)
- Use HTTPS only in production

## Script Status

All scripts support both JSON2 and JSON-RPC protocols automatically:
- ✅ auth_json2.sh - Fixed UID variable, correct endpoint format
- ✅ auth_jsonrpc.sh - Working
- ✅ search_records.sh - Updated for JSON2
- ✅ create_record.sh - Updated for JSON2
- ✅ update_record.sh - Updated for JSON2
- ✅ delete_record.sh - Updated for JSON2
- ✅ execute_method.sh - Updated for JSON2
- ✅ install_module.sh - Updated for JSON2
- ✅ uninstall_module.sh - Updated for JSON2
- ✅ upgrade_module.sh - Updated for JSON2

## For Claude Code

When using this skill:

1. **Read SKILL.md first** for usage guidance
2. **Consult JSON2_ENDPOINT_FORMAT.md** for JSON2 implementation details
3. **Always request credentials** from user (never hardcode)
4. **Check ODOO_PROTOCOL** environment variable to determine active protocol
5. **Use EXAMPLES_JSON2.md** for practical code examples

## References

- [Odoo 19.0 JSON2 API](https://www.odoo.com/documentation/19.0/developer/reference/external_api.html)
- [Odoo 18.0 JSON-RPC](https://www.odoo.com/documentation/18.0/developer/howtos/web_services.html)
