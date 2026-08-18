# API Contract Reference (schema.yaml)

## Full Schema Example

```yaml
openapi: 3.0.0
info:
  version: 1.0.0                      # API version (1.0.0 becomes v1 in URL)
  title: My Custom API
  x-sfcc-endpoint-class: shopper      # MANDATORY (project discipline). 'shopper' or 'admin'.
components:
  securitySchemes:
    ShopperToken:                     # For Shopper APIs (requires siteId)
      type: oauth2
      flows:
        clientCredentials:
          tokenUrl: https://{shortCode}.api.commercecloud.salesforce.com/shopper/auth/v1/organizations/{organizationId}/oauth2/token
          scopes:
            c_my_scope: Description of my scope
    AmOAuth2:                         # For Admin APIs (no siteId)
      type: oauth2
      flows:
        clientCredentials:
          tokenUrl: https://account.demandware.com/dwsso/oauth2/access_token
          scopes:
            c_my_admin_scope: Description of my admin scope
  parameters:
    siteId:
      name: siteId
      in: query
      required: true
      schema:
        type: string
        minLength: 1
    locale:
      name: locale
      in: query
      required: false
      schema:
        type: string
        minLength: 1
paths:
  /my-endpoint:
    get:
      summary: Get something
      operationId: getMyData         # Must match function name in script
      parameters:
        - $ref: '#/components/parameters/siteId'
        - in: query
          name: c_my_param           # Custom params must start with c_
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Success
          content:
            application/json:
              schema:
                type: object
security:
  - ShopperToken: ['c_my_scope']     # Global security (or per-operation)
```

## `servers` URL and live SCAPI paths

Shopper Custom API requests are served under **`/custom/{apiFolderName}/`** where `{apiFolderName}` is the **directory name** under `cartridge/rest-apis/` (lowercase, hyphens). In OpenAPI 3 **`servers`** entries, base URLs must include that segment (for example `.../custom/my-api-name/`). Mismatched paths (for example a vanity `/my-brand/...` prefix) break client generation and cause **404** or registration failures even when `api.json` and `script.js` look correct.

## Contract Requirements

- **Version:** Defined in `info.version`, transformed to URL version (e.g., `1.0.1` becomes `v1`)
- **Security Scheme:** Use `ShopperToken` for Shopper APIs or `AmOAuth2` for Admin APIs
- **Custom Scopes:** Must match the platform allowlist `^c_[A-Za-z0-9._-]{1,23}$` (≤ 25 chars including the `c_` prefix). Projects may adopt a stricter internal convention (e.g. `^c_[a-z0-9]{1,23}$`) for consistency.
- **Parameters:** All request parameters must be defined; **custom query params** must have `c_` prefix. `siteId` / `locale` are system parameters and stay unprefixed. **Request/response body fields are NOT required to carry `c_`** — the prefix discipline applies to query parameters only (the platform reserves the un-prefixed query-param namespace).
- **System Parameters:** `siteId` and `locale` must have `type: string` and `minLength: 1`
- **Request body `additionalProperties` MUST NOT be present (empirical):** Custom Shopper SCAPI request body schemas MUST NOT include the `additionalProperties` key at all (any value, including `false`). On code activation the instance `error-*.log` shows: `"Invalid API schema. The key 'additionalProperties' is restricted and can not be used"` followed by `java.lang.NullPointerException` in `CustomApiRegistry.registerEndpointCountGauges` — a fatal registry crash that drops ALL Custom API endpoints platform-wide. Symptom: SCAPI dispatcher returns 404 "Custom API request couldn't be translated" for every endpoint until removal. Working schemas that register correctly have zero `additionalProperties` keys. Always verify SCAPI registration after `b2c code activate`: `b2c logs get --search 'CustomApiRegistry|ScapiDispatcherServlet' --level ERROR` — any rows = registry crashed.
- **`x-sfcc-endpoint-class: shopper | admin` declaration MANDATORY (project discipline):** gates security-scheme selection and scope discipline.

## Shopper vs Admin APIs

| Aspect | Shopper API | Admin API |
|--------|-------------|-----------|
| Security Scheme | `ShopperToken` | `AmOAuth2` |
| `siteId` Parameter | Required | Must omit |
| Max Runtime | 10 seconds | 60 seconds |
| Max Request Body | 5 MiB | 20 MB |
| Activity Type | STOREFRONT | BUSINESS_MANAGER |

## Path Parameter Example

```yaml
paths:
  /items/{itemId}:
    get:
      operationId: getItem
      parameters:
        - $ref: '#/components/parameters/siteId'
        - in: path
          name: itemId
          required: true
          schema:
            type: string
```

## Request Body Example (POST/PUT/PATCH)

```yaml
paths:
  /items:
    post:
      operationId: createItem
      parameters:
        - $ref: '#/components/parameters/siteId'
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              # DO NOT add `additionalProperties` — restricted by Custom Shopper SCAPI platform.
              # Adding it (any value) crashes CustomApiRegistry with NPE; ALL endpoints drop to 404.
              required:
                - name
              properties:
                name:
                  type: string
                c_customField:               # `c_` prefix optional in body (query-only rule)
                  type: string
      responses:
        '201':
          description: Created
```

If your request body uses a `$ref` to a shared component schema, the **referenced** schema also must NOT contain `additionalProperties`. Embedded `oneOf` / `anyOf` branches inherit the same restriction — every branch must omit `additionalProperties`. Strict request-body validation must rely on `required` + property `type` constraints alone.
