# B2C Business Manager Roles & Permissions Skill

Use the `b2c` CLI to manage **Business Manager (BM)** instance-level roles, user assignments, and permissions.

> **Account Manager vs Business Manager:** This skill covers the **BM plane** — roles that live on a specific Commerce Cloud instance. For **Account Manager** users, AM roles (e.g. `bm-admin`), organizations, and API clients, use the `b2c_am` skill. The two planes are separate: AM controls who can log in and their org/tenant grants; BM controls per-instance role definitions and permissions.

> **Tip:** If `b2c` is not installed globally, use `npx @salesforce/b2c-cli` instead.

## Overview

| Area | Topic | Description |
|------|-------|-------------|
| Business Manager | `bm roles` | List, create, delete instance-level BM roles |
| Business Manager | `bm roles grant/revoke` | Assign/unassign users to BM roles on an instance |
| Business Manager | `bm roles permissions` | Get/set role permissions on an instance |

## Business Manager Roles

BM role commands operate on a specific Commerce Cloud instance (via `--server` or config).

```bash
# list BM roles on the configured instance
b2c bm roles list

# target a different instance
b2c bm roles list --server my-sandbox.demandware.net

# get role details (with user list)
b2c bm roles get Administrator --expand users

# create a custom role
b2c bm roles create MyCustomRole --description "Custom role for content editors"

# delete a custom role (system roles cannot be deleted)
b2c bm roles delete MyCustomRole

# grant a BM role to a user on the instance
b2c bm roles grant user@example.com --role Administrator

# revoke a BM role from a user
b2c bm roles revoke user@example.com --role Administrator

# all commands support --json for machine-readable output
b2c bm roles list --json
```

## Business Manager Role Permissions

Permissions use a file-based get/set workflow since the API replaces all permissions at once.

```bash
# view permission summary
b2c bm roles permissions get Administrator

# export permissions to a JSON file for editing
b2c bm roles permissions get Administrator --output admin-perms.json

# edit the file, then apply
b2c bm roles permissions set Administrator --file admin-perms.json
```

The permissions JSON has four sections: `functional`, `module`, `locale`, and `webdav`. Each can be scoped to organization, site, or unscoped depending on type.

## Authentication Requirements

| Operations | Client Credentials | User Auth |
|---|---|---|
| BM Roles | OCAPI permissions for `/roles` resource | OCAPI permissions for `/roles` resource |

## Related Skills

- `b2c_am` - Account Manager users, AM roles, organizations, and API clients (the AM plane)
- `b2c_config` - Configure authentication credentials and instance settings
- `b2c_sandbox` - Create and manage sandboxes (instances)
