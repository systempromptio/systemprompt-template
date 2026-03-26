---
name: "Odoo Pilot"
description: "Control and manage Odoo instances through API. Supports JSON-RPC and JSON2 protocols for authentication, CRUD, and module management"
---

-|-------------|----------------|
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
