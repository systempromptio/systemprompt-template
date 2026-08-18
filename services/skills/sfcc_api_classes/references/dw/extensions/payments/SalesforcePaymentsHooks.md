# dw.extensions.payments.SalesforcePaymentsHooks

## Overview
Interface for registering script hooks that customize Salesforce Payments behavior. Defines extension point names and hook function signatures.

## Description
Provides extension points for asynchronous payment processing. Hook implementations must export functions from a cartridge script and register them in the site's hooks.json.

```ts
declare class SalesforcePaymentsHooks {
    /** The extension point name for async payment success. */
    static extensionPointAsyncPaymentSucceeded: 'dw.extensions.payments.asyncPaymentSucceeded'

    /**
     * Called when asynchronous payment succeeded for the given order.
     * @param order - the order whose asynchronous payment succeeded
     * @returns Status|null - returning a non-null result ends hook execution and is ignored
     */
    asyncPaymentSucceeded(order: dw.order.Order): dw.system.Status
}
```
