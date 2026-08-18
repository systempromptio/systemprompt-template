# dw.customer.CustomerGroup

## Overview
Represents a customer segmentation group used to apply different site experiences, pricing, or promotions.

## Description
CustomerGroup represents either explicit or rule-based (dynamic) groups of customers. Explicit groups contain an explicit list of registered customers; dynamic groups determine membership via business rules.

```ts
declare class CustomerGroup extends dw.object.ExtensibleObject {
    /**
     * Description text for the group (read-only).
     */
    description: string

    /**
     * Unique semantic ID of the customer group (read-only).
     */
    ID: string

    /**
     * True if group membership is rule-based (read-only).
     */
    ruleBased: boolean

    /**
     * Assigns the specified registered customer to this explicit group.
     * @param customer - registered Customer, must not be null
     */
    assignCustomer(customer: Customer): void

    /**
     * Returns the description of the customer group.
     */
    getDescription(): string

    /**
     * Returns the unique ID of the customer group.
     */
    getID(): string

    /**
     * Returns true if the group is rule-based.
     */
    isRuleBased(): boolean

    /**
     * Unassigns the specified registered customer from this explicit group.
     * @param customer - registered Customer, must not be null
     */
    unassignCustomer(customer: Customer): void
}
```
