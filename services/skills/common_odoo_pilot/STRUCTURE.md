# Odoo Pilot - Structure & Organization

This document explains the organization and purpose of each file in the odoo-pilot skill.

## Directory Structure

```
odoo-pilot/
├── Documentation Files (7 total)
│   ├── SKILL.md                    # Main entry point
│   ├── README.md                   # Quick reference
│   ├── STRUCTURE.md                # This file
│   ├── PROTOCOL_GUIDE.md           # Protocol details
│   ├── JSON2_ENDPOINT_FORMAT.md    # JSON2 format reference
│   ├── EXAMPLES_JSON2.md           # Practical examples
│   └── CHANGELOG_JSON2.md          # Implementation history
│
└── scripts/ (12 scripts)
    ├── Authentication (4)
    │   ├── auth.sh                 # Unified authentication (RECOMMENDED)
    │   ├── auth_json2.sh           # JSON2 authentication (legacy)
    │   ├── auth_jsonrpc.sh         # JSON-RPC authentication (legacy)
    │   └── detect_version.sh       # Auto-detect Odoo version
    │
    ├── CRUD Operations (5)
    │   ├── search_records.sh       # Search/read records
    │   ├── create_record.sh        # Create records
    │   ├── update_record.sh        # Update records
    │   ├── delete_record.sh        # Delete records
    │   └── execute_method.sh       # Execute custom methods
    │
    └── Module Management (3)
        ├── install_module.sh       # Install modules
        ├── uninstall_module.sh     # Uninstall modules
        └── upgrade_module.sh       # Upgrade modules
```

## File Purposes

### Documentation Files

#### 1. SKILL.md (Main Documentation)
**Purpose**: Main entry point for the skill. Contains complete usage guide.

**When to read**: First time using the skill, or as general reference.

**Contents**:
- Quick start guide
- All script usage examples
- Common workflows
- Error handling
- Security best practices
- Model reference table

#### 2. README.md (Quick Reference)
**Purpose**: Quick reference card for fast lookups.

**When to read**: When you need a quick reminder of structure or basic usage.

**Contents**:
- Directory structure overview
- Documentation guide (what to read when)
- Quick start commands
- Critical JSON2 format reminder
- Script status checklist

#### 3. STRUCTURE.md (This File)
**Purpose**: Explains the organization and purpose of each file.

**When to read**: To understand how the skill is organized and which files to use.

**Contents**:
- Directory structure
- File purposes and usage
- Reading recommendations
- Cross-references

#### 4. PROTOCOL_GUIDE.md (Protocol Comparison)
**Purpose**: Comprehensive guide on JSON-RPC vs JSON2 protocols.

**When to read**:
- When choosing between protocols
- When migrating from JSON-RPC to JSON2
- When troubleshooting protocol issues

**Contents**:
- Protocol comparison table
- When to use each protocol
- Environment variable setup
- Migration guide
- Security best practices
- Troubleshooting section

#### 5. JSON2_ENDPOINT_FORMAT.md (JSON2 Format Reference)
**Purpose**: CRITICAL reference for correct JSON2 API format.

**When to read**:
- ALWAYS when implementing JSON2 requests
- When debugging JSON2 errors
- When writing new JSON2 code

**Contents**:
- Correct endpoint structure (`/json/2/{model}/{method}`)
- Request/response examples for all operations
- Headers required (Authorization, X-Odoo-Database)
- Body format (direct parameters, no wrapper)
- cURL examples
- Comparison with JSON-RPC format

**CRITICAL**: This file documents the correct JSON2 format after fixing the initial incorrect implementation.

#### 6. EXAMPLES_JSON2.md (Practical Examples)
**Purpose**: Working examples for common operations.

**When to read**: When implementing specific operations or workflows.

**Contents**:
- CRUD operation examples
- Module management examples
- Complex workflows (search→update, bulk operations, migration)
- Domain syntax guide
- Troubleshooting common errors

#### 7. CHANGELOG_JSON2.md (Implementation History)
**Purpose**: Technical documentation of JSON2 implementation and fixes.

**When to read**:
- When understanding why code is structured this way
- When debugging implementation issues
- For historical context

**Contents**:
- All changes made to implement JSON2
- Before/after code comparisons
- Bug fixes (UID variable conflict, endpoint format)
- Technical explanations
- Testing notes

### Script Files

All scripts are located in `scripts/` and are executable. They support both JSON2 and JSON-RPC protocols automatically.

#### Authentication Scripts

**auth.sh [protocol]** (RECOMMENDED - Unified Authentication)
- Automatically detects best protocol (JSON2 first, fallback to JSON-RPC)
- Can force specific protocol: `json2` or `jsonrpc`
- Tries JSON2 authentication, falls back to JSON-RPC if needed
- Sets appropriate variables based on protocol used
- Usage:
  - `eval $(./scripts/auth.sh)` - Auto-detect
  - `eval $(./scripts/auth.sh json2)` - Force JSON2
  - `eval $(./scripts/auth.sh jsonrpc)` - Force JSON-RPC

**auth_json2.sh** (Legacy - use auth.sh instead)
- Authenticates using Bearer token (Odoo >= 19.0)
- Sets ODOO_UID, ODOO_VERSION, ODOO_PROTOCOL
- Usage: `eval $(./scripts/auth_json2.sh)`

**auth_jsonrpc.sh** (Legacy - use auth.sh instead)
- Authenticates using session cookie (Odoo < 19.0)
- Sets ODOO_SESSION_ID, ODOO_UID, ODOO_VERSION
- Usage: `eval $(./scripts/auth_jsonrpc.sh)`

**detect_version.sh**
- Auto-detects Odoo version and available protocols
- No authentication required
- Usage: `./scripts/detect_version.sh`

#### CRUD Operation Scripts

**search_records.sh**
- Search and read records from any model
- Usage: `./scripts/search_records.sh <model> <domain> [fields] [limit] [offset]`

**create_record.sh**
- Create new record in any model
- Usage: `./scripts/create_record.sh <model> <values_json>`
- Returns: New record ID

**update_record.sh**
- Update existing record(s)
- Usage: `./scripts/update_record.sh <model> <record_ids> <values_json>`

**delete_record.sh**
- Delete record(s) from model
- Usage: `./scripts/delete_record.sh <model> <record_ids>`
- Warning: Permanent deletion

**execute_method.sh**
- Execute any method on any model
- Usage: `./scripts/execute_method.sh <model> <method> <args> [kwargs]`

#### Module Management Scripts

**install_module.sh**
- Install Odoo module by name
- Usage: `./scripts/install_module.sh <module_name>`

**uninstall_module.sh**
- Uninstall Odoo module by name
- Usage: `./scripts/uninstall_module.sh <module_name>`
- Warning: May delete data

**upgrade_module.sh**
- Upgrade existing module
- Usage: `./scripts/upgrade_module.sh <module_name>`

## Reading Recommendations

### For First-Time Users
1. Read [SKILL.md](SKILL.md) - Quick Start section
2. Set environment variables
3. Run `detect_version.sh` and authenticate
4. Try basic operations from [SKILL.md](SKILL.md) examples

### For JSON2 Implementation
1. Read [JSON2_ENDPOINT_FORMAT.md](JSON2_ENDPOINT_FORMAT.md) - **CRITICAL**
2. Understand endpoint structure and headers
3. Review [EXAMPLES_JSON2.md](EXAMPLES_JSON2.md) for practical examples
4. Consult [PROTOCOL_GUIDE.md](PROTOCOL_GUIDE.md) for migration details

### For Troubleshooting
1. Check [PROTOCOL_GUIDE.md](PROTOCOL_GUIDE.md) - Troubleshooting section
2. Review [EXAMPLES_JSON2.md](EXAMPLES_JSON2.md) - Error solutions
3. Check [CHANGELOG_JSON2.md](CHANGELOG_JSON2.md) - Known issues

### For Claude Code Agents
When using this skill, Claude should:
1. **CRITICAL**: Understand version-protocol mapping:
   - Odoo >= 19.0 → MUST use JSON2
   - Odoo <= 18.0 → MUST use JSON-RPC
2. Read [SKILL.md](SKILL.md) for general usage
3. Consult [JSON2_ENDPOINT_FORMAT.md](JSON2_ENDPOINT_FORMAT.md) for JSON2 format (Odoo >= 19.0)
4. Use [EXAMPLES_JSON2.md](EXAMPLES_JSON2.md) for code examples
5. Always request credentials from user (never hardcode)
6. Check `ODOO_PROTOCOL` environment variable to determine active protocol
7. Never try to use JSON2 with Odoo <= 18.0
8. Never try to use JSON-RPC with Odoo >= 19.0 (unless for compatibility testing)

## Key Concepts

### Protocol Selection by Version (CRITICAL)

**Version-based protocol rules - NO EXCEPTIONS:**
- **Odoo >= 19.0**: MUST use JSON2 protocol
- **Odoo <= 18.0**: MUST use JSON-RPC protocol

The protocol is NOT a choice - it's determined by the Odoo version.

### Dual Protocol Support
All scripts automatically detect and use the correct protocol based on `ODOO_PROTOCOL` environment variable set by authentication:
- `json2` - For Odoo >= 19.0 ONLY (stateless, Bearer token)
- `jsonrpc` - For Odoo <= 18.0 ONLY (session-based, cookie)

**Version-Protocol Mapping:**
```
Odoo 8.x - 18.x  → JSON-RPC (mandatory)
Odoo 19.x+       → JSON2 (mandatory)
Odoo 20.x+       → JSON2 only (JSON-RPC removed)
```

### Security
- **NEVER hardcode credentials** in any file
- All documentation uses placeholder values (`YOUR_API_KEY_HERE`, `your-instance.odoo.com`)
- Scripts always read credentials from environment variables
- User must provide credentials at runtime

### JSON2 Critical Points
The JSON2 implementation was corrected from an initial incorrect implementation:

**INCORRECT** (old):
- Endpoint: `/json/2/call`
- Body: JSON-RPC wrapper with params

**CORRECT** (current):
- Endpoint: `/json/2/{model}/{method}`
- Headers: `Authorization: bearer {API_KEY}`, `X-Odoo-Database: {DB}`
- Body: Direct parameters (no wrapper)

See [JSON2_ENDPOINT_FORMAT.md](JSON2_ENDPOINT_FORMAT.md) for complete details.

## File Cross-References

- [SKILL.md](SKILL.md) references all other documentation files
- [README.md](README.md) provides quick links to key sections
- [PROTOCOL_GUIDE.md](PROTOCOL_GUIDE.md) references [JSON2_ENDPOINT_FORMAT.md](JSON2_ENDPOINT_FORMAT.md)
- [EXAMPLES_JSON2.md](EXAMPLES_JSON2.md) references [PROTOCOL_GUIDE.md](PROTOCOL_GUIDE.md)
- [CHANGELOG_JSON2.md](CHANGELOG_JSON2.md) references [JSON2_ENDPOINT_FORMAT.md](JSON2_ENDPOINT_FORMAT.md)

## Status

### Completed
✅ All scripts updated for JSON2 support
✅ All documentation created and sanitized
✅ Dual protocol support implemented
✅ Security requirements met (no hardcoded credentials)
✅ Backward compatibility with JSON-RPC maintained
✅ Directory structure cleaned (empty folders removed)

### Not Tested
⚠️ Scripts have NOT been tested against production environment per user request
⚠️ Testing should be done in development/staging environment first

## Maintenance Notes

When updating this skill:
1. Always maintain dual protocol support
2. Never commit credentials to documentation
3. Update [CHANGELOG_JSON2.md](CHANGELOG_JSON2.md) with significant changes
4. Keep [JSON2_ENDPOINT_FORMAT.md](JSON2_ENDPOINT_FORMAT.md) as the authoritative JSON2 format reference
5. Ensure all scripts remain executable (`chmod +x scripts/*.sh`)

---

Last Updated: 2026-01-27
