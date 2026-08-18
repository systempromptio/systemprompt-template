# dw.customer.CustomerList

## Overview
Represents the collection of customers registered for a given site.

## Description
A CustomerList is the container of registered customers for a site. Every site has one customer list, though lists may be shared across sites.

```ts
declare class CustomerList  {
    /**
     * Optional description of the customer list (read-only).
     */
    description: string

    /**
     * ID of the customer list (read-only).
     */
    ID: string

    /**
     * Returns the description of the list.
     */
    getDescription(): string

    /**
     * Returns the ID of the customer list.
     */
    getID(): string
}
```
