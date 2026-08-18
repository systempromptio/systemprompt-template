# dw.customer.CustomerContextMgr

## Overview
Helper methods for managing customer context, primarily the effective shopping time associated with a customer.

## Description
Provides helper methods for managing customer context, such as the effective time for which the customer is shopping.

```ts
declare class CustomerContextMgr  {
    /**
     * Effective time associated with the customer (nullable).
     */
    effectiveTime: Date

    /**
     * Get the effective time associated with the customer. When null, no effective time is set.
     */
    static getEffectiveTime(): Date

    /**
     * Set the effective time for the customer. Pass null to remove the effective time.
     * @param effectiveTime - the new effective Date or null
     */
    static setEffectiveTime(effectiveTime: Date): void
}
```
