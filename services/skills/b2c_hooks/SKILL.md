# B2C Commerce Hooks

Hooks are extension points that allow you to customize business logic by registering scripts. B2C Commerce supports two types of hooks:

1. **OCAPI/SCAPI Hooks** - Extend API resources with before, after, and modifyResponse hooks
2. **System Hooks** - Custom extension points for order calculation, payment, and other core functionality

## Hook Types Overview

| Type | Purpose | Examples |
|------|---------|----------|
| OCAPI/SCAPI | Extend API behavior | `dw.ocapi.shop.basket.afterPOST` |
| System | Core business logic | `dw.order.calculate` |
| Custom | Your own extension points | `app.checkout.validate` |

## Hook Registration

### File Structure

```
my_cartridge/
├── package.json           # References hooks.json
└── cartridge/
    └── scripts/
        ├── hooks.json     # Hook registrations
        └── hooks/         # Hook implementations
            ├── basket.js
            └── order.js
```

### package.json

Reference the hooks configuration file:

```json
{
  "name": "my_cartridge",
  "hooks": "./cartridge/scripts/hooks.json"
}
```

### hooks.json

Register hooks with their implementing scripts:

```json
{
  "hooks": [
    {
      "name": "dw.ocapi.shop.basket.afterPOST",
      "script": "./hooks/basket.js"
    },
    {
      "name": "dw.ocapi.shop.basket.modifyPOSTResponse",
      "script": "./hooks/basket.js"
    },
    {
      "name": "dw.order.calculate",
      "script": "./hooks/order.js"
    }
  ]
}
```

### Hook Script

Export functions matching the hook method name (without package prefix):

```javascript
// hooks/basket.js
exports.afterPOST = function(basket) {
    // Called after basket creation.
    // Returning undefined lets the hook chain continue to other cartridges.
    // Return new Status(Status.OK) only if you intentionally need to stop the chain.
};

exports.modifyPOSTResponse = function(basket, basketResponse) {
    // Modify the API response
    basketResponse.c_customField = 'value';
};
```

## HookMgr API

Use `dw.system.HookMgr` to call hooks programmatically:

```javascript
var HookMgr = require('dw/system/HookMgr');

// Check if hook exists
if (HookMgr.hasHook('dw.order.calculate')) {
    // Call the hook
    var result = HookMgr.callHook('dw.order.calculate', 'calculate', basket);
}
```

| Method | Description |
|--------|-------------|
| `hasHook(extensionPoint)` | Returns true if hook is registered or has default implementation |
| `callHook(extensionPoint, functionName, args...)` | Calls the hook, returns result or undefined |

## Status Object

Hooks return `dw.system.Status` to indicate success or failure:

```javascript
var Status = require('dw/system/Status');

// Error - stop processing, rollback transaction
var status = new Status(Status.ERROR);
status.addDetail('error_code', 'INVALID_ADDRESS');
status.addDetail('message', 'Address validation failed');
return status;
```

| Return value | HTTP Response | Behavior |
|--------|---------------|----------|
| `undefined` (no return) | Continues | **Chain continues** to next cartridge — preferred for OCAPI/SCAPI hooks |
| `Status.OK` | Continues | **Chain STOPS** — no further cartridges execute for this hook. Use only in system hooks (`dw.order.calculate`, payment, etc.) or when you are certain this is the only/last cartridge for this hook |
| `Status.ERROR` | 400 Bad Request | Transaction rolled back, processing stops |
| Uncaught exception | 500 Internal Error | Transaction rolled back |

**CRITICAL — OCAPI/SCAPI chain semantics:** For `dw.ocapi.*` / SCAPI hooks, `return new Status(Status.OK)` **terminates the hook chain** — every cartridge registered after yours on the cartridge path is silently skipped. This is contrary to what Salesforce Infocenter implies. To let the chain continue, return nothing (`undefined`). Only `Status.ERROR` should be returned explicitly (to abort the request). This is a common source of real-world incidents (e.g. wrong cart totals because `dw.order.calculate` was skipped).

## OCAPI/SCAPI Hooks

OCAPI and SCAPI share the same hooks. Enable in Business Manager:
**Administration > Global Preferences > Feature Switches > Enable Salesforce Commerce Cloud API hook execution**

### Hook Types

| Hook | When Called | Use Case |
|------|-------------|----------|
| `before<METHOD>` | Before processing | Validation, access control |
| `after<METHOD>` | After processing (in transaction) | Data modification, external calls |
| `modify<METHOD>Response` | Before response sent | Add/modify response properties |

### Common Hook Patterns

```javascript
// Validation in beforePUT
exports.beforePUT = function(basket, addressDoc) {
    if (!isValidAddress(addressDoc)) {
        var status = new Status(Status.ERROR);
        status.addDetail('validation_error', 'Invalid address');
        return status;
    }
};

// External call in afterPOST (within transaction)
exports.afterPOST = function(basket, paymentDoc) {
    var result = callPaymentService(paymentDoc);
    request.custom.paymentResult = result; // Pass to modifyResponse
    // No return — let the chain continue to other cartridges.
};

// Modify response
exports.modifyPOSTResponse = function(basket, basketResponse, paymentDoc) {
    basketResponse.c_paymentStatus = request.custom.paymentResult.status;
};
```

### Passing Data Between Hooks

Use `request.custom` to pass data between hooks in the same request:

```javascript
// In afterPOST
exports.afterPOST = function(basket, doc) {
    request.custom.externalId = callExternalService();
};

// In modifyPOSTResponse
exports.modifyPOSTResponse = function(basket, response, doc) {
    response.c_externalId = request.custom.externalId;
};
```

### Detect SCAPI vs OCAPI

```javascript
exports.afterPOST = function(basket) {
    if (request.isSCAPI()) {
        // SCAPI-specific logic
    } else {
        // OCAPI-specific logic
    }
};
```

## System Hooks

### Calculate Hooks

| Extension Point | Function | Purpose |
|-----------------|----------|---------|
| `dw.order.calculate` | `calculate` | Full basket/order calculation |
| `dw.order.calculateShipping` | `calculateShipping` | Shipping calculation |
| `dw.order.calculateTax` | `calculateTax` | Tax calculation |

```javascript
// hooks/calculate.js
var Status = require('dw/system/Status');
var HookMgr = require('dw/system/HookMgr');

exports.calculate = function(lineItemCtnr) {
    // Calculate shipping
    HookMgr.callHook('dw.order.calculateShipping', 'calculateShipping', lineItemCtnr);

    // Calculate promotions, totals...

    // Calculate tax
    HookMgr.callHook('dw.order.calculateTax', 'calculateTax', lineItemCtnr);

    return new Status(Status.OK);
};
```

### Payment Hooks

| Extension Point | Function | Purpose |
|-----------------|----------|---------|
| `dw.order.payment.authorize` | `authorize` | Payment authorization |
| `dw.order.payment.capture` | `capture` | Capture authorized payment |
| `dw.order.payment.refund` | `refund` | Refund payment |
| `dw.order.payment.validateAuthorization` | `validateAuthorization` | Check authorization validity |
| `dw.order.payment.reauthorize` | `reauthorize` | Re-authorize expired auth |

### Order Hooks

| Extension Point | Function | Purpose |
|-----------------|----------|---------|
| `dw.order.createOrderNo` | `createOrderNo` | Custom order number generation |

```javascript
var OrderMgr = require('dw/order/OrderMgr');
var Site = require('dw/system/Site');

exports.createOrderNo = function() {
    var seqNo = OrderMgr.createOrderSequenceNo();
    var prefix = Site.current.ID;
    return prefix + '-' + seqNo;
};
```

## Custom Hooks

Create your own extension points:

```javascript
// Define custom hook
var HookMgr = require('dw/system/HookMgr');

function processCheckout(basket) {
    // Call custom hook if registered
    if (HookMgr.hasHook('app.checkout.validate')) {
        var status = HookMgr.callHook('app.checkout.validate', 'validate', basket);
        if (status && status.error) {
            return status;
        }
    }
    // Continue processing...
}
```

Register in hooks.json:

```json
{
  "hooks": [
    {
      "name": "app.checkout.validate",
      "script": "./hooks/checkout.js"
    }
  ]
}
```

Custom hooks always execute all registered implementations regardless of return value.

## Remote Includes in Hooks

Enhance API responses with data from other SCAPI endpoints:

```javascript
var RESTResponseMgr = require('dw/system/RESTResponseMgr');

exports.modifyGETResponse = function(product, doc) {
    // Include Custom API response
    var include = RESTResponseMgr.createScapiRemoteInclude(
        'custom',           // API family
        'my-api',           // API name
        'v1',               // Version
        'endpoint'          // Endpoint
    );
    doc.c_additionalData = { value: [include] };
};
```

## Before Adding an OCAPI/SCAPI Hook

**Mandatory checklist** before registering a new `dw.ocapi.*` hook in `hooks.json`:

1. **Search for existing registrations:** `grep -r "dw.ocapi.shop.<resource>.<method>" sfcc/cartridges/*/hooks.json sfcc/cartridges/*/cartridge/scripts/hooks.json` — list every cartridge that already registers this hook name.
2. **Check cartridge path order:** Open the site's `custom-cartridges` in `site.xml` (or BM → Sites → Manage Sites → Cartridges). Your cartridge's position relative to others determines who runs first.
3. **Do NOT return `Status.OK`:** For OCAPI/SCAPI hooks, `return new Status(Status.OK)` silently kills the chain. Let the function return `undefined`. Only return `Status.ERROR` to abort the request.
4. **If other cartridges exist for the same hook:** Verify your logic doesn't conflict. If you need data from a later cartridge, consider using `request.custom` or reordering.

## Best Practices

### Do

- Let OCAPI/SCAPI hook functions return `undefined` (no explicit return) to preserve the chain
- Return `Status.OK` only in system hooks (`dw.order.calculate`, payment hooks) where it is expected
- Use `request.custom` to pass data between hooks
- Check `request.isSCAPI()` when supporting both APIs
- Keep hooks focused and performant
- Use custom properties (`c_` prefix) in modifyResponse
- Search for existing hook registrations before adding a new one

### Don't

- **Return `Status.OK` from OCAPI/SCAPI hooks** — it stops the chain (return `undefined` to let it continue)
- Use transactions in calculate hooks (breaks SCAPI)
- Modify standard response properties (only `c_` properties)
- Rely on hook execution order across cartridges without checking
- Make slow external calls in beforeGET (affects caching)

## Error Handling

### Circuit Breaker

Too many hook errors triggers circuit breaker (HTTP 503):

```json
{
  "title": "Hook Circuit Breaker",
  "type": "https://api.commercecloud.salesforce.com/.../hook-circuit-breaker",
  "detail": "Failure rate above threshold of '50%'",
  "extensionPointName": "dw.ocapi.shop.basket.afterPOST"
}
```

### Timeout

Hooks must complete within the SCAPI timeout (HTTP 504 on timeout).

## Detailed References

- [OCAPI/SCAPI Hooks](references/OCAPI-SCAPI-HOOKS.md) - API hook patterns and available hooks
- [System Hooks](references/SYSTEM-HOOKS.md) - Calculate, payment, and order hooks
