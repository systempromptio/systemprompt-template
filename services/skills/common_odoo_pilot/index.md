
# Odoo Pilot

Control your Odoo instances through API calls with ease. This skill provides automated scripts and guidance for managing Odoo databases, modules, and records via HTTP API.

## Protocol Selection Rules

**CRITICAL - Protocol selection is version-based:**
- **Odoo >= 19.0**: ALWAYS use JSON2 protocol
- **Odoo <= 18.0**: ALWAYS use JSON-RPC protocol

The `auth.sh` script automatically detects the version and selects the appropriate protocol.

## Documentation Index

- **[README.md](README.md)** - Quick reference guide
- **[STRUCTURE.md](STRUCTURE.md)** - File organization & purpose guide
- **[PROTOCOL_GUIDE.md](PROTOCOL_GUIDE.md)** - Protocol comparison & migration guide
- **[JSON2_ENDPOINT_FORMAT.md](JSON2_ENDPOINT_FORMAT.md)** - JSON2 format reference (CRITICAL for Odoo >= 19)
- **[EXAMPLES_JSON2.md](EXAMPLES_JSON2.md)** - Practical usage examples
- **[CHANGELOG_JSON2.md](CHANGELOG_JSON2.md)** - Implementation details & fixes

## Quick Start

### 1. Set Environment Variables

Before using any script, configure your Odoo credentials:

```bash
export ODOO_URL="https://your-instance.odoo.com"
export ODOO_DB="your_database"
export ODOO_USER="admin"
export ODOO_KEY="your-api-key-or-password"
```

**Security Note**: Never hardcode credentials. Always use environment variables or secure credential storage.

### 2. Authenticate

```bash
# Auto-detect protocol and authenticate
eval $(./scripts/auth.sh)

# Or force specific protocol
eval $(./scripts/auth.sh json2)      # Force JSON2 (Odoo >= 19)
eval $(./scripts/auth.sh jsonrpc)    # Force JSON-RPC (Odoo < 19)
```

### 3. Start Working

```bash
# Search for companies
./scripts/search_records.sh res.partner '[["is_company","=",true]]' '["name","email"]' 5

# Install a module
./scripts/install_module.sh sale_management

# Create a record
./scripts/create_record.sh res.partner '{"name":"New Company","email":"info@company.com"}'
```

## Available Scripts

### Authentication & Detection

#### `auth.sh [protocol]`
**Unified authentication script** that automatically detects and uses the best protocol.

```bash
# Auto-detect protocol (tries JSON2 first, falls back to JSON-RPC)
eval $(./scripts/auth.sh)

# Force specific protocol
eval $(./scripts/auth.sh json2)      # Force JSON2 (Odoo >= 19)
eval $(./scripts/auth.sh jsonrpc)    # Force JSON-RPC (Odoo < 19)
```

**Sets**:
- JSON2: `ODOO_UID`, `ODOO_VERSION`, `ODOO_PROTOCOL='json2'`, `ODOO_AUTH_HEADER`
- JSON-RPC: `ODOO_SESSION_ID`, `ODOO_UID`, `ODOO_VERSION`, `ODOO_PROTOCOL='jsonrpc'`

**Requirements**:
- Always: `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`
- JSON-RPC only: `ODOO_USER`

#### `detect_version.sh`
Auto-detect Odoo version and protocol support (optional, for information only).

```bash
./scripts/detect_version.sh
```

**Output**: Displays detected Odoo version, protocol type, and endpoints.

#### Legacy Authentication Scripts (Deprecated)

`auth_json2.sh` and `auth_jsonrpc.sh` are still available for compatibility but **`auth.sh` is now recommended** as it handles both protocols automatically.

### Module Management

#### `install_module.sh <module_name>`
Install an Odoo module.

```bash
./scripts/install_module.sh sale_management
./scripts/install_module.sh stock
```

#### `uninstall_module.sh <module_name>`
Uninstall an Odoo module.

```bash
./scripts/uninstall_module.sh sale_management
```

#### `upgrade_module.sh <module_name>`
Upgrade an existing module.

```bash
./scripts/upgrade_module.sh account
```

### CRUD Operations

#### `search_records.sh <model> <domain> [fields] [limit] [offset]`
Search and read records from any model.

```bash
# Search companies with specific fields
./scripts/search_records.sh res.partner '[["is_company","=",true]]' '["name","email","phone"]' 10

# Search all sale orders
./scripts/search_records.sh sale.order '[]' '["name","state","amount_total"]' 50

# Search with offset (pagination)
./scripts/search_records.sh product.product '[["sale_ok","=",true]]' '["name","list_price"]' 20 40
```

**Domain Syntax**: Odoo uses Polish notation for domains:
- Single condition: `[["field","=","value"]]`
- Multiple AND: `[["field1","=","a"],["field2",">",10]]`
- OR condition: `["|",["field1","=","a"],["field2","=","b"]]`
- NOT condition: `["!",["field","=","value"]]`

#### `create_record.sh <model> <values_json>`
Create a new record in any model.

```bash
# Create a partner
./scripts/create_record.sh res.partner '{"name":"John Doe","email":"john@example.com","phone":"+34600000000"}'

# Create a product
./scripts/create_record.sh product.product '{"name":"New Product","list_price":99.99,"type":"consu"}'
```

**Returns**: ID of the newly created record.

#### `update_record.sh <model> <record_ids> <values_json>`
Update existing record(s).

```bash
# Update single record
./scripts/update_record.sh res.partner '[123]' '{"phone":"+34612345678","mobile":"+34699999999"}'

# Update multiple records
./scripts/update_record.sh product.product '[1,2,3]' '{"list_price":150.00}'
```

#### `delete_record.sh <model> <record_ids>`
Delete record(s) from a model.

```bash
# Delete single record
./scripts/delete_record.sh res.partner '[123]'

# Delete multiple records
./scripts/delete_record.sh product.product '[1,2,3]'
```

**Warning**: Deletion is permanent. Ensure you have backups.

### Generic Method Execution

#### `execute_method.sh <model> <method> <args> [kwargs]`
Execute any method on any Odoo model.

```bash
# Get name representation of records
./scripts/execute_method.sh res.partner name_get '[[1,2,3]]' '{}'

# Confirm a sale order
./scripts/execute_method.sh sale.order action_confirm '[[5]]' '{}'

# Get system parameter
./scripts/execute_method.sh ir.config_parameter get_param '["web.base.url"]' '{}'

# Send email
./scripts/execute_method.sh mail.mail send '[[123]]' '{}'
```

## Protocol Differences

**Version-Based Protocol Selection:**
- **Odoo >= 19.0**: MUST use JSON2
- **Odoo <= 18.0**: MUST use JSON-RPC

### JSON-RPC (Odoo <= 18.0)
- **Use for**: Odoo versions 8.0 through 18.0
- **Endpoint**: `/jsonrpc` and `/web/session/authenticate`
- **Session Management**: Required (cookie-based)
- **Authentication**: Must authenticate once, then use session cookie
- **Structure**: Uses `execute_kw` wrapper

```bash
# For Odoo <= 18.0 only
eval $(./scripts/auth_jsonrpc.sh)

# Session persists via ODOO_SESSION_ID
./scripts/search_records.sh res.partner '[]'
```

### JSON2 (Odoo >= 19.0)
- **Use for**: Odoo versions 19.0 and later
- **Endpoint**: `/json/2/{model}/{method}` (NOT `/json/2/call`)
- **Session Management**: Not required (stateless)
- **Authentication**: Bearer token in `Authorization` header
- **Structure**: Direct parameters in body, no JSON-RPC wrapper
- **Headers**: `Authorization: bearer {API_KEY}`, `X-Odoo-Database: {DB_NAME}`

```bash
# For Odoo >= 19.0 only
eval $(./scripts/auth_json2.sh)

# API key automatically included in each request via Bearer token
./scripts/search_records.sh res.partner '[]'
```

**Recommended**: Use `auth.sh` for automatic protocol selection based on Odoo version.

**Important**: For detailed JSON2 format documentation, see:
- `JSON2_ENDPOINT_FORMAT.md` - Complete endpoint reference with examples
- `PROTOCOL_GUIDE.md` - Protocol comparison and migration guide
- `EXAMPLES_JSON2.md` - Practical usage examples
- `CHANGELOG_JSON2.md` - Implementation details and fixes

## Common Workflows

### Workflow 1: Install Multiple Modules

```bash
# Set credentials
export ODOO_URL="https://demo.odoo.com"
export ODOO_DB="demo"
export ODOO_KEY="your-api-key"
export ODOO_USER="admin"  # Only needed for JSON-RPC

# Authenticate (auto-detects protocol)
eval $(./scripts/auth.sh)

# Install modules
./scripts/install_module.sh sale_management
./scripts/install_module.sh purchase
./scripts/install_module.sh stock
```

### Workflow 2: Bulk Create Partners

```bash
# Authenticate
eval $(./scripts/auth.sh)

# Create multiple partners
for name in "Company A" "Company B" "Company C"; do
    ./scripts/create_record.sh res.partner "{\"name\":\"${name}\",\"is_company\":true}"
done
```

### Workflow 3: Search and Update

```bash
# Find all products with price < 50
RESULTS=$(./scripts/search_records.sh product.product '[["list_price","<",50]]' '["id"]' 1000)

# Extract IDs (requires jq)
IDS=$(echo "$RESULTS" | jq '[.[].id]')

# Update all prices by 10%
./scripts/update_record.sh product.product "$IDS" '{"list_price":55.00}'
```

### Workflow 4: Module Lifecycle Management

```bash
# Check current modules
./scripts/search_records.sh ir.module.module '[["state","=","installed"]]' '["name","state"]'

# Upgrade a module
./scripts/upgrade_module.sh sale

# Uninstall if needed
./scripts/uninstall_module.sh sale
```

## Error Handling

All scripts follow consistent error handling:

1. **Exit Codes**:
   - `0` = Success
   - `1` = Error (check stderr for message)

2. **Output Streams**:
   - `stdout` = Data output (JSON, IDs, results)
   - `stderr` = Status messages and errors

3. **Error Messages**: Always include context and suggested solutions

```bash
# Capture both output and errors
RESULT=$(./scripts/search_records.sh res.partner '[]' 2>&1)
EXIT_CODE=$?

if [[ $EXIT_CODE -ne 0 ]]; then
    echo "Error occurred: $RESULT"
fi
```

## Common Odoo Models

| Model | Description | Common Methods |
|-------|-------------|----------------|
| `res.partner` | Contacts/Companies | search_read, create, write |
| `res.users` | Users | create, write, has_group |
| `sale.order` | Sales Orders | action_confirm, action_cancel |
| `purchase.order` | Purchase Orders | button_confirm, button_cancel |
| `product.product` | Products | search_read, create, write |
| `product.template` | Product Templates | create, write |
| `account.move` | Invoices/Journal Entries | action_post, button_draft |
| `stock.picking` | Inventory Transfers | action_confirm, button_validate |
| `ir.module.module` | Modules | button_immediate_install |
| `ir.config_parameter` | System Parameters | get_param, set_param |

## Security Best Practices

1. **API Keys**: Use API keys instead of passwords (Odoo >= 13.0)
   - Navigate to: Settings > Users & Companies > Users > API Keys

2. **Dedicated Users**: Create integration users with minimal permissions

3. **Environment Variables**: Never commit credentials to version control

4. **HTTPS Only**: Always use HTTPS in production

5. **Audit Logging**: Monitor API access through Odoo audit logs

6. **Rate Limiting**: Implement delays between bulk operations

## Troubleshooting

### Authentication Fails
```
Error: Authentication failed: Access Denied
```
**Solutions**:
- Verify credentials are correct
- Check if API keys are enabled (Odoo >= 13.0)
- Ensure user has API access permissions

### Module Not Found
```
Error: Module 'xyz' not found
```
**Solutions**:
- Check module name spelling
- Update module list: Apps > Update Apps List
- Verify module is available in your Odoo version

### Permission Denied
```
Error: You do not have permission to...
```
**Solutions**:
- Grant appropriate access rights to user
- Check user groups and permissions
- Use admin user for testing

### Connection Timeout
```
Error: Could not connect to Odoo instance
```
**Solutions**:
- Verify ODOO_URL is correct
- Check network connectivity
- Ensure Odoo instance is running

## Tips for Claude Code Usage

When using this skill, follow these guidelines:

1. **Always authenticate first**: Run detect_version and auth scripts before operations

2. **Use eval for auth scripts**: Auth scripts output environment variables, use `eval $(...)`

3. **Check exit codes**: Scripts return 0 on success, 1 on error

4. **Parse JSON output**: Use `jq` or similar for JSON manipulation

5. **Handle errors gracefully**: Check stderr for error messages

6. **Batch operations wisely**: Add delays between bulk operations to avoid rate limiting

7. **Test on demo first**: Use Odoo demo instances for testing workflows

8. **Read documentation files**: For detailed protocol information, consult:
   - `JSON2_ENDPOINT_FORMAT.md` for correct JSON2 request/response formats
   - `PROTOCOL_GUIDE.md` for protocol selection and migration guidance
   - `EXAMPLES_JSON2.md` for practical working examples

9. **Request credentials**: Never hardcode credentials. Always ask the user for:
   - `ODOO_URL` - Instance URL
   - `ODOO_DB` - Database name
   - `ODOO_KEY` - API key (for JSON2) or password (for JSON-RPC)
   - `ODOO_USER` - Username (only for JSON-RPC)

## API Documentation References

### Official Odoo Documentation
- **JSON2 API (Odoo 19.0+)**: https://www.odoo.com/documentation/19.0/developer/reference/external_api.html
- **JSON-RPC (Legacy)**: https://www.odoo.com/documentation/18.0/developer/howtos/web_services.html
- **Model Reference**: https://www.odoo.com/documentation/18.0/developer/reference/backend/orm.html

### Skill Documentation Files
- **`JSON2_ENDPOINT_FORMAT.md`**: Comprehensive JSON2 endpoint format reference with curl examples
- **`PROTOCOL_GUIDE.md`**: Complete protocol comparison, migration guide, and troubleshooting
- **`EXAMPLES_JSON2.md`**: Practical examples for CRUD operations, module management, and workflows
- **`CHANGELOG_JSON2.md`**: Technical details of JSON2 implementation and fixes applied

## Integration with Other Skills

This skill works well with:
- **odoo-importation**: For bulk data imports from XLSX/CSV
- **Data transformation skills**: For preparing data before import
- **Reporting skills**: For extracting and analyzing Odoo data
