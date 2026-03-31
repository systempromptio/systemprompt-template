# Changelog - JSON2 Bearer Token Implementation

## Date: 2026-01-27

### Summary

Updated all Odoo Pilot scripts to support proper JSON2 authentication using Bearer tokens as specified in the official Odoo 19.0 documentation. This ensures compatibility with Odoo 19+ and prepares for the deprecation of XML-RPC in Odoo 20.0.

### Changes Made

#### 1. Authentication Script
**File**: `scripts/auth_json2.sh`

**Changes**:
- ✅ Fixed `UID` variable conflict by renaming to `ODOO_UID`
- ✅ Implemented Bearer token authentication via `Authorization: bearer` header
- ✅ Removed old context-based authentication (db/login/password in params)
- ✅ Added proper error handling for 404 (endpoint not available)
- ✅ Simplified authentication flow by calling `res.users.search_read` to verify token
- ✅ Export `ODOO_AUTH_HEADER` environment variable for easy reuse

**Authentication Method**:
```bash
curl -H "Authorization: bearer ${ODOO_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"call","params":{...}}'
```

#### 2. CRUD Operation Scripts

All CRUD scripts updated to use Bearer token in JSON2 mode:

**Files Updated**:
- ✅ `scripts/search_records.sh`
- ✅ `scripts/create_record.sh`
- ✅ `scripts/update_record.sh`
- ✅ `scripts/delete_record.sh`
- ✅ `scripts/execute_method.sh`

**Changes per script**:
1. Added API key validation check for JSON2 protocol
2. Removed context-based authentication (db/login/password)
3. Added `Authorization: bearer ${ODOO_KEY}` header
4. Simplified params structure (removed nested context object)

**Before** (incorrect):
```json
{
  "params": {
    "context": {
      "db": "...",
      "login": "...",
      "password": "..."
    },
    "model": "...",
    "method": "..."
  }
}
```

**After** (correct):
```json
{
  "params": {
    "model": "...",
    "method": "..."
  }
}
```
With header: `Authorization: bearer ${ODOO_KEY}`

#### 3. Module Management Scripts

**Files Updated**:
- ✅ `scripts/install_module.sh`
- ✅ `scripts/uninstall_module.sh`
- ✅ `scripts/upgrade_module.sh`

**Changes**:
- Same Bearer token implementation as CRUD scripts
- Removed context authentication
- Added API key validation
- Simplified request structure

#### 4. Documentation

**New Files**:
- ✅ `PROTOCOL_GUIDE.md` - Comprehensive guide on JSON-RPC vs JSON2
- ✅ `CHANGELOG_JSON2.md` - This file

**Documentation Includes**:
- Protocol comparison table
- Environment variable reference
- Quick start guides for both protocols
- API key generation instructions
- Migration guide from JSON-RPC to JSON2
- Security best practices
- Troubleshooting section
- Complete usage examples

### Technical Details

#### JSON2 API Authentication (Odoo 19.0+)

**Endpoint**: `/json/2/call`

**Authentication**: Bearer token (API Key)
- API keys must be generated in: Settings > Users > Account Security > API Keys
- Maximum duration: 3 months
- Recommended: Short duration (1 day) for security

**Request Structure**:
```bash
POST /json/2/call HTTP/1.1
Host: your-instance.odoo.com
Authorization: bearer YOUR_API_KEY_HERE
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "method": "call",
  "params": {
    "model": "res.partner",
    "method": "search_read",
    "args": [[["is_company","=",true]]],
    "kwargs": {"fields": ["name","email"], "limit": 10}
  },
  "id": 1
}
```

**Key Differences from JSON-RPC**:
1. **Stateless**: No session cookies required
2. **Bearer Token**: API key in Authorization header, not in request body
3. **Simplified Params**: No db/uid/password in params structure
4. **Direct Method Call**: No service wrapper (object/common/db)

#### Environment Variables

**For JSON2** (required):
```bash
export ODOO_URL="https://your-instance.odoo.com"
export ODOO_DB="your_database"
export ODOO_KEY="your-api-key"  # Must be API key, not password
export ODOO_PROTOCOL="json2"     # Set by auth_json2.sh
```

**For JSON-RPC** (legacy):
```bash
export ODOO_URL="https://your-instance.odoo.com"
export ODOO_DB="your_database"
export ODOO_USER="user@example.com"
export ODOO_KEY="api-key-or-password"
export ODOO_PROTOCOL="jsonrpc"   # Set by auth_jsonrpc.sh
```

### Migration Path

#### From JSON-RPC (Odoo 8-18) to JSON2 (Odoo 19+)

**Step 1**: Generate API Keys
```
Settings > Users & Companies > Users > [Your User] > Account Security > New API Key
```

**Step 2**: Update Authentication
```bash
# Old
export ODOO_USER="user@example.com"
export ODOO_KEY="password"
eval $(./scripts/auth_jsonrpc.sh)

# New
export ODOO_KEY="generated-api-key"
eval $(./scripts/auth_json2.sh)
```

**Step 3**: No changes needed to other script calls
```bash
# These work with both protocols automatically
./scripts/search_records.sh res.partner '[]' '["name"]' 10
./scripts/create_record.sh res.partner '{"name":"Test"}'
./scripts/install_module.sh sale_management
```

### Backward Compatibility

✅ **All scripts maintain full backward compatibility**

- Scripts detect protocol via `$ODOO_PROTOCOL` environment variable
- Default to `jsonrpc` if not set
- Both JSON-RPC and JSON2 code paths remain functional
- No breaking changes to script interfaces

### Testing Notes

⚠️ **IMPORTANT**: Scripts were NOT tested against production environment per user request.

**When testing**:
1. Test on development/staging environment first
2. Verify API key has correct permissions
3. Check JSON2 endpoint availability: `curl https://your-instance/json/2/call`
4. Monitor Odoo logs for authentication issues
5. Verify CRUD operations work as expected

### Known Issues & Limitations

1. **JSON2 Endpoint Availability**
   - Some Odoo 19 installations may not have `/json/2/call` enabled
   - Error: 404 Not Found
   - Solution: Use JSON-RPC (still works in v19) or enable JSON2

2. **API Key Permissions**
   - API keys inherit user permissions
   - Insufficient permissions may cause "Access Denied" errors
   - Solution: Ensure user has appropriate access rights

3. **API Key Expiration**
   - Keys expire based on configured duration
   - No automatic renewal
   - Solution: Monitor expiration and regenerate before expiry

### Security Improvements

✅ **Enhanced Security**:
1. Bearer token authentication (industry standard)
2. Stateless design (no session hijacking risk)
3. API keys separate from passwords
4. Short-lived keys recommended (1 day)
5. No credentials in request body
6. HTTPS required (enforced)

### Performance Considerations

**JSON2 Benefits**:
- ✅ Stateless (no session management overhead)
- ✅ Faster for microservices/serverless architectures
- ✅ Better for distributed systems
- ✅ No cookie parsing overhead
- ✅ Direct method calls (no service wrapper)

**JSON-RPC Limitations**:
- ❌ Session state management required
- ❌ Cookie handling complexity
- ❌ Session expiration issues
- ❌ Not ideal for stateless apps
- ❌ Being deprecated in v20.0

### Future Work

**Recommendations**:
1. Test all scripts thoroughly in staging environment
2. Create automated test suite for both protocols
3. Add protocol auto-detection based on Odoo version
4. Implement automatic API key rotation helper
5. Add monitoring for API key expiration
6. Create migration checklist for v20.0 (XML-RPC removal)

### References

- [Odoo 19.0 JSON2 API Documentation](https://www.odoo.com/documentation/19.0/developer/reference/external_api.html)
- [Odoo API 101: What's new in Odoo 19](https://oduist.com/blog/odoo-experience-2025-ai-summaries-2/177-odoo-api-101-how-does-it-work-and-what-s-new-in-odoo-19-179)
- [Odoo Forum: JSON API and API Keys](https://www.odoo.com/forum/help-1/json-api-api-keys-and-user-connection-292915)

### Contributors

- David Gómez (ENTERPRISE DEMO)
- Claude Sonnet 4.5 (AI Assistant)

### License

Same as parent project (Odoo Pilot skill)

---

## Quick Reference Card

### JSON2 Authentication Flow
```bash
# 1. Setup
export ODOO_URL="https://instance.odoo.com"
export ODOO_DB="database"
export ODOO_KEY="api-key-here"

# 2. Authenticate
eval $(./scripts/auth_json2.sh)

# 3. Use any script
./scripts/search_records.sh res.partner '[]' '["name"]' 5
```

### Verify Bearer Token Works
```bash
curl -X POST "https://your-instance/json/2/call" \
  -H "Authorization: bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "call",
    "params": {
      "model": "res.users",
      "method": "search_read",
      "args": [[], ["id","name"]],
      "kwargs": {"limit": 1}
    },
    "id": 1
  }'
```

### Common Error Solutions

| Error | Cause | Solution |
|-------|-------|----------|
| 404 Not Found | JSON2 endpoint not available | Use JSON-RPC or enable JSON2 |
| Access Denied | Invalid/expired API key | Regenerate API key |
| Authentication Failed | Using password instead of API key | Generate and use API key |
| Missing Authorization | Bearer token not sent | Check ODOO_KEY variable |
