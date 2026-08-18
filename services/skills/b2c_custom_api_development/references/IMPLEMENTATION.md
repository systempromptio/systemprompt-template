# Implementation Reference (script.js)

## Complete Implementation Example

```javascript
var RESTResponseMgr = require('dw/system/RESTResponseMgr');

exports.getMyData = function() {
    // Get query parameters
    var myParam = request.getHttpParameterMap().get('c_my_param').getStringValue();

    // Get path parameters (for paths like /items/{itemId})
    var itemId = request.getSCAPIPathParameters().get('itemId');

    // Get request body (for POST/PUT/PATCH)
    var requestBody = JSON.parse(request.httpParameterMap.requestBodyAsString);

    // Business logic here...
    var result = {
        data: 'my data',
        param: myParam
    };

    // Return success response
    RESTResponseMgr.createSuccess(result).render();
};
exports.getMyData.public = true;  // Required: mark function as public

// Error response example
exports.getMyDataWithError = function() {
    RESTResponseMgr
        .createError(404, 'not-found', 'Resource Not Found', 'The requested resource was not found.')
        .render();
};
exports.getMyDataWithError.public = true;
```

## Best Practices

- Always return JSON format responses
- Use RFC 9457 error format with at least the `type` field
- Mark all exported functions with `.public = true`
- Handle errors gracefully to avoid circuit breaker activation
- GET requests cannot commit transactions

## Caching Responses

The only **verified-safe** cache directive for Custom Shopper SCAPI is `response.setExpires(...)`. The SCAPI gateway server-side cache layer (JWA) honours `setExpires` and serves the cached body without invoking the handler on hits.

```javascript
// Cache for 60 seconds (JWA / server-side; this is what actually caches)
response.setExpires(Date.now() + 60000);
```

**Restrictions verified on a production sandbox:**

- The SCAPI gateway **blocks** scripts from setting **`Cache-Control`** via `response.setHttpHeader('Cache-Control', ...)` — throws `IllegalArgumentException: Header name Cache-Control is not allowed to be set or added.` and logs once per request. Do **not** call it.
- The platform forces `Cache-Control: no-store` on every Custom Shopper SCAPI response on the wire, so the **shared cache / eCDN does NOT cache** custom endpoints. Public TTLs must be configured at eCDN, not in script.
- **`response.setVaryBy('...')` in Custom Shopper SCAPI handlers is unverified.** It works in classic SFRA controllers but its behaviour on Custom SCAPI is **not documented in the official Custom API guide**. Use only after empirical sandbox verification for your endpoint, and only when the cached layer is JWA-side (eCDN ignores it anyway because of the `no-store` override).

**Caches actually in play on a Custom Shopper SCAPI response:**

| Layer | Caches what | TTL source | Notes |
|---|---|---|---|
| eCDN / CloudFront | nothing (Cache-Control: no-store forced by platform) | — | Configure shared cache at eCDN, not in script. |
| JWA (server-side web-adapter cache) | the full response body | `response.setExpires(...)` | On hit, handler is **not invoked**; `retrievedAt`-style timestamps stay frozen. Only reliable hit-tell. |
| Custom named cache (`dw/system/CacheMgr`) | per-cartridge data (e.g., query results) | `caches.json` `expireAfterSeconds` | Per-POD, in-memory. Cleared on code deploy. Manual flush in BM → Administration → Operations → Custom Caches. |

## Rhino / Java-bridged objects (`dw.*`)

Property access on Java-bridged Script API objects does **not** behave like plain JavaScript: reading a **non-existent** property can **throw** instead of yielding `undefined`. Do **not** use `typeof obj.someMethod === 'function'` to probe optional APIs on `dw.*` instances. Wrap the access and call in **`try { ... } catch (_e) { ... }`** (or use version-gated code paths documented for your compatibility mode). Unit tests that mock `dw.*` with plain JS objects may miss this — validate on a sandbox when touching optional Script API members.

## Remote Includes

Include responses from other SCAPI endpoints:

```javascript
var include = dw.system.RESTResponseMgr.createScapiRemoteInclude(
    'custom', 'other-api', 'v1', 'endpointPath',
    dw.web.URLParameter('siteId', 'MySite')
);

var response = {
    data: 'my data',
    included: [include]
};
RESTResponseMgr.createSuccess(response).render();
```

## External Service Calls

When calling external services via `LocalServiceRegistry.createService()`, configure the service in Business Manager or import via site archive:

```javascript
var LocalServiceRegistry = require('dw/svc/LocalServiceRegistry');

var service = LocalServiceRegistry.createService('my.external.api', {
    createRequest: function(svc, args) {
        svc.setRequestMethod('GET');
        svc.addHeader('Authorization', 'Bearer ' + args.token);
        return null;
    },
    parseResponse: function(svc, client) {
        return JSON.parse(client.text);
    }
});

var result = service.call({ token: 'my-token' });
```

See [b2c-webservices](mdc:.cursor/skills/build/b2c-webservices/SKILL.md) skill for service configuration and services.xml format.

## services.xml Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<services xmlns="http://www.demandware.com/xml/impex/services/2014-09-26">

    <service-credential service-credential-id="my.external.api">
        <url>https://api.example.com/v1</url>
    </service-credential>

    <service-profile service-profile-id="my.external.api.profile">
        <timeout-millis>5000</timeout-millis>
        <rate-limit-enabled>false</rate-limit-enabled>
        <cb-enabled>true</cb-enabled>
        <cb-calls>5</cb-calls>
        <cb-millis>10000</cb-millis>
    </service-profile>

    <service service-id="my.external.api">
        <service-type>HTTP</service-type>
        <enabled>true</enabled>
        <log-prefix>MYAPI</log-prefix>
        <comm-log-enabled>true</comm-log-enabled>
        <profile-id>my.external.api.profile</profile-id>
        <credential-id>my.external.api</credential-id>
    </service>

</services>
```

Import with: `b2c job import ./my-services-folder`

## HTTP Methods Supported

- GET (no transaction commits)
- POST
- PUT
- PATCH
- DELETE
- HEAD
- OPTIONS

## Circuit Breaker Protection

Custom APIs have a circuit breaker that blocks requests when error rate exceeds 50%:

1. Circuit opens after 50+ errors in 100 requests
2. Requests return 503 for 60 seconds
3. Circuit enters half-open state, testing next 10 requests
4. If >5 fail, circuit reopens; otherwise closes

**Prevention:** Write robust code with error handling and avoid long-running remote calls.
