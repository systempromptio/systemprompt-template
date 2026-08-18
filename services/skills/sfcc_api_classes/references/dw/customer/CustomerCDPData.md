# dw.customer.CustomerCDPData

## Overview
Read-only view of a customer's Salesforce CDP (Customer Data Platform) data: segments and emptiness check.

## Description
Provides access to CDP-derived segments for a customer and a utility to test whether any meaningful CDP data exists. Intended for read-only inspection; refer to Salesforce CDP docs for enablement and semantics.

```ts
declare class CustomerCDPData  {
    /** True if the CDP data contains no meaningful content. */
    readonly empty: boolean

    /** Read-only array of segment identifiers for the customer. */
    readonly segments: string[]

    /** Returns an array containing the CDP segments for the customer. */
    getSegments(): string[]

    /** Returns true if the CDPData is empty (no meaningful data). */
    isEmpty(): boolean
}
```
