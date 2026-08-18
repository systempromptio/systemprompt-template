# B2C SCAPI Custom APIs Skill

Use the `b2c` CLI plugin to manage SCAPI Custom API endpoints and check their registration status.

> **Tip:** If `b2c` is not installed globally, use `npx @salesforce/b2c-cli` instead (e.g., `npx @salesforce/b2c-cli scapi custom status`).

## Required: Tenant ID

The `--tenant-id` flag is **required** for all commands. The tenant ID identifies your B2C Commerce instance.

**Important:** The tenant ID is NOT the same as the organization ID:

- **Tenant ID**: `zzxy_prd` (used with commands that require `--tenant-id`)
- **Organization ID**: `f_ecom_zzxy_prd` (used in SCAPI URLs, has `f_ecom_` prefix)

### Deriving Tenant ID from Hostname

For sandbox instances, you can derive the tenant ID from the hostname by replacing hyphens with underscores:

| Hostname                                   | Tenant ID  |
| ------------------------------------------ | ---------- |
| `zzpq-013.dx.commercecloud.salesforce.com` | `zzpq_013` |
| `zzxy-001.dx.commercecloud.salesforce.com` | `zzxy_001` |
| `abcd-dev.dx.commercecloud.salesforce.com` | `abcd_dev` |

For production instances, use your realm and instance identifier (e.g., `zzxy_prd`).

## Examples

### Get Custom API Endpoint Status

```bash
# list all Custom API endpoints for an organization
b2c scapi custom status --tenant-id zzxy_prd

# list with JSON output
b2c scapi custom status --tenant-id zzxy_prd --json
```

### Filter by Status

```bash
# list only active endpoints
b2c scapi custom status --tenant-id zzxy_prd --status active

# list only endpoints that failed to register
b2c scapi custom status --tenant-id zzxy_prd --status not_registered
```

### Group by Type or Site

```bash
# group endpoints by API type (Admin vs Shopper)
b2c scapi custom status --tenant-id zzxy_prd --group-by type

# group endpoints by site
b2c scapi custom status --tenant-id zzxy_prd --group-by site
```

### Customize Output Columns

```bash
# show extended columns (includes error reasons, sites, etc.)
b2c scapi custom status --tenant-id zzxy_prd --extended

# select specific columns to display
b2c scapi custom status --tenant-id zzxy_prd --columns type,apiName,status,sites

# available columns: type, apiName, apiVersion, cartridgeName, endpointPath, httpMethod, status, sites, securityScheme, operationId, schemaFile, implementationScript, errorReason, id
```

### Debug Failed Registrations

```bash
# quickly find and diagnose failed Custom API registrations
b2c scapi custom status --tenant-id zzxy_prd --status not_registered --columns type,apiName,endpointPath,errorReason
```

### Configuration

The tenant ID and short code can be set via environment variables:

- `SFCC_TENANT_ID`: Tenant ID (e.g., `zzxy_prd`, not the organization ID)
- `SFCC_SHORTCODE`: SCAPI short code

### More Commands

See `b2c scapi custom --help` for a full list of available commands and options.

## Cartridge development workflow

When implementing B2C Commerce cartridge code (custom SCAPI cartridges `int_*_scapi`, integration cartridges `int_*`, BM cartridges `bm_*`, `services.xml`, `hooks.json`, `steptypes.json`, or any `dw.*` script API):

**Hard rules:**
- Never invent `dw.*` classes, OCAPI/SCAPI endpoints, hook IDs, or XSD elements. Verify via the `b2c_docs` skill or the skill matching the task (`b2c_hooks`, `b2c_custom_api_development`, `b2c_custom_job_steps`, `b2c_webservices`, etc.).
- Custom SCAPI conventions: scopes `c_<feature>` (≤25 chars), RFC 7807 problem helpers, `setExpires`-only caching (no `Cache-Control`, no `Vary`), BM allowlist via Custom Objects, `meta-config.json` auto/manual XML split, AM scope grant, BM cartridge-path requirement. See the `b2c_custom_api_development` skill.
- Follow the project's commit convention (see the `git_commit` skill). No secrets in commits (`dw.json`, tokens, `.env`).

**Design-quality discipline.** Apply extra care when any of these triggers apply: (a) introducing a new service in `services.xml` or a new hook in `hooks.json`; (b) adding a new custom SCAPI family or endpoint family; (c) splitting/merging a controller, job step, or hook handler; (d) editing more than 3 cartridge files in one change; (e) the task uses the words "clean up", "extract", "decouple", "abstract".

When a trigger applies, reason explicitly about:
- **SOLID** — especially for service wrappers (LocalServiceRegistry adapters), hook fan-out, and decorator chains.
- **Design patterns** — name the pattern (Strategy for payment processor selection, Adapter for external services, Observer for hook chains, Template Method for decorators, Chain of Responsibility for middleware).
- **Refactoring** — cite the catalog technique in commit/PR text (Extract Function, Replace Conditional with Polymorphism, Move Method, Introduce Parameter Object).

If none apply, note it as a mechanical change with no design impact.

**Workflow:**
1. Identify the right `b2c_*` skill and read it before writing code.
2. If a design trigger applies, name the principle/pattern before editing.
3. Use the `b2c` CLI (via the `b2c_code` / `b2c_job` / `b2c_logs` / `b2c_sandbox` skills) for deploys, logs, jobs, and sandbox operations — never guess flags.
4. For unfamiliar runtime shapes, write a small script and run it via `b2c code`, or check logs via the `b2c_logs` skill.

Do not commit, push, or open PRs unless explicitly asked.

## Related Skills

- `b2c_custom_api_development` - Creating Custom API endpoints (schema, script, mapping)
- `b2c_code` - Deploying and activating code versions (triggers registration)
