# SFCC General Development Skill

## JavaScript Engine & Scripting Model

- Java-based platform using Rhino JS engine
- **Compatibility mode & ES support:** The instance's compatibility mode determines the Rhino engine version and JS language level. Older modes (e.g. 18.10) run Rhino **1.7R5** with ES6 **not** enabled (`VERSION_ES6` unset), so server cartridge code must be **ES5-shaped**: use `function` expressions, no arrow functions, no destructuring, no template literals, no shorthand object properties/methods, no spread/rest, no `for...of`, no `class`. `const`/`let` are OK **outside** loop bodies only (Mozilla-style; use `var` inside `while`/`for`). Newer compatibility modes enable ES6+ features. Confirm your instance's compatibility mode before relying on modern syntax, and align your server-side ESLint config accordingly.
- **No direct Java class access** — `java.lang.*`, `java.util.*`, `Packages.*` etc. are NOT available. Only `dw.*` APIs are exposed. Never use `java.lang.Thread.sleep()`, `java.util.Date`, or any raw Java classes. For delays use busy-wait (`while (Date.now() < end) {}`)
- Fully synchronous — no promises, async/await, setTimeout, or microtasks
- CommonJS modules (require, module.exports, Modules 1.1.1 spec)
- `JSON.stringify` fails on Java objects — convert to plain JS first
- **JSDoc:** `@param`/`@returns` type annotations are OK on `module.exports` (no TS inference in `.js`). Avoid narrative comment blocks that just restate the code.
- API usage details: see `sfcc_api_classes` skill

## `require()` Path Resolution

- **`./` or `../`**: relative to current module. Extension optional.
- **`*/`**: resolves to first cartridge having same path (cartridge path order from site.xml)
- **`~/`**: relative to current cartridge
- **`dw/`**: SFCC built-in APIs (e.g., `require('dw/util/HashMap')`)
- **No prefix**: top-level module

**File extensions** (priority): `.js`, `.ds`, `.json`

Tips:
- `module.superModule`: finds same filename in next cartridge in path
- Don't use `require` in global scope unless for `dw/` modules or middlewares
- Place `require` inside functions for performance

## SFCC Core Concepts

- **Cartridges**: Modular code units loaded in defined order (cartridge path)
- **dw.* Namespace**: Root for all server-side SFCC classes
- **ExtensibleObject**: Base for objects with custom attributes
- **PersistentObject**: Base for database-stored objects (Customer, Product)

### Key Namespaces

| Namespace | Contains |
|-----------|----------|
| `dw.catalog.*` | Catalog, Product, Category, PriceBook, Inventory |
| `dw.campaign.*` | Campaign, Promotion, Discount, Coupon |
| `dw.customer.*` | Customer, Profile, AddressBook, CustomerGroup |
| `dw.order.*` | Order, Basket, LineItem, Shipment, Payment |
| `dw.content.*` | Content, Library, Folder, Page |
| `dw.system.*` | Status, Logger, Transaction, Site, HookMgr |
| `dw.util.*` | Collection, ArrayList, HashMap, Calendar, UUID |
| `dw.io.*` | File, FileReader, FileWriter, CSVStreamReader |
| `dw.svc.*` | Service, ServiceRegistry, Result |
| `dw.web.*` | URLUtils, URLRedirect, Resource |

### Global Objects

- `request` — dw.system.Request
- `response` — dw.system.Response
- `session` — dw.system.Session
- `customer` — dw.customer.Customer
- `empty` — checks null/undefined/empty collection (avoid in modern code)

## Frequently Used Classes

**Catalog:** `ProductMgr` (getProduct, queryAllProducts), `Product` (.ID, .name, .priceModel, .custom, getMasterProduct, getVariationModel), `Category` (.ID, .displayName, getOnlineSubCategories), `CatalogMgr`, `ProductSearchModel`

**Campaign:** `PromotionMgr` (getActivePromotions, applyDiscounts), `Promotion`, `DiscountPlan`, `CouponMgr`

**Customer/Order:** `CustomerMgr` (getCustomerByLogin, createCustomer), `Customer` (.profile, .addressBook, .custom), `OrderMgr` (createOrder, getOrder), `Basket` (.productLineItems, addProduct, getTotalGrossPrice, getShipments)

**Content:** `ContentMgr` (getContent, getFolder), `PageMgr` (getPage, getPageType)

**System:** `Transaction` (wrap, begin, commit, rollback), `Status` (.OK, .ERROR, getStatus, getMessage), `Logger` (getLogger, .info, .warn, .error), `Site` (getCurrent, getCustomPreferenceValue), `HookMgr`

**File/IO:** `File`, `FileReader`, `FileWriter`, `CSVStreamReader`

## Collections & Iterators

- SFCC uses its own Collection, ArrayList, Iterator, HashMap types
- Common methods: `forEach`, `map`, `find`, `reduce`, `concat`, `first`, `every`
- `toArray()` for small collections (10-20 items); use iterators directly for large ones
- Prefer native Map/Array over SFCC HashMap/ArrayList when possible

## Patterns & Examples

```js
// Require & Transaction
var ProductMgr = require('dw/catalog/ProductMgr');
var Transaction = require('dw/system/Transaction');

Transaction.wrap(function() {
  basket.custom.flag = true;
});

// Iterator
var items = someCollection.iterator();
while (items.hasNext()) {
  var item = items.next();
}

// Custom Attribute (safe check)
var value = 'myAttribute' in obj.custom && obj.custom.myAttribute;

// Logging
var Logger = require('dw/system/Logger');
Logger.getLogger('integration', 'custom').debug('Payload: {0}', JSON.stringify(payload));

// Status pattern
var Status = require('dw/system/Status');
function doSomething() {
  if (/* error */) return new Status(Status.ERROR, 'ERROR_CODE', 'Message');
  return new Status(Status.OK);
}

// Service call
var service = LocalServiceRegistry.createService('my.http', {/* callbacks */});
var result = service.call(requestObj);
if (result.status === Status.OK) {
  // handle response
} else {
  Logger.error('Service failed: ' + result.errorMessage);
}

// Site preferences
var pref = require('dw/system/Site').getCurrent().getCustomPreferenceValue('myPref');

// Product search
var psm = new (require('dw/catalog/ProductSearchModel'))();
psm.setSearchPhrase('*');
psm.search();

// Find customer by email
var c = CustomerMgr.queryProfile('email ILIKE {0}', 'user@example.com').next();
```

## Product & Variant Handling

- Products: simple, master, variant, bundle, or set
- Use `getMasterProduct()`, `getVariationModel()`, `getVariants()` to traverse
- `ProductVariationModel`: `getVariationAttributes()`, `getAllValues()`, `getSelectedValue()`
- Bundles/Sets: `getBundledProducts()`, `getProductSetProducts()`
- Inventory: `getAvailabilityModel()`, `ProductInventoryMgr`

## Logging

- Pattern: `Logger.getLogger(category, logFile)`
- Levels: `debug`, `info`, `warn`, `error`, `fatal`
- Log types: System logs, Custom logs (customdebug, custominfo, customwarn, customerror)
- Service logs: `service-[prefix]-[id]-[date].log` when comm logs enabled

## Quotas & Limits

SFCC enforces platform constraints. When an **enforced quota** is exceeded, an exception is thrown. **Never catch quota exceptions** — design to avoid violations.

Common violations: too many in-memory collection elements, too many persistent objects.

## Troubleshooting

- Always wrap writes in `Transaction.wrap`
- Check custom attributes safely: `'attr' in obj.custom`
- Service failures return `Status.ERROR` — always check before using results
- Multi-site: always use `Site.getCurrent()` for context
- Script timeout is 30 seconds
