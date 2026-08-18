# dw.customer.ProductListRegistrant

## Overview
Holds information about a person associated with an event-related product list (for example a gift registry). Stores name, email and role for the registrant.

## Description
A ProductListRegistrant represents a person linked to an event product list (such as a bride or groom). Provides accessors and mutators for the registrant's email, first name, last name and role.

```ts
declare class ProductListRegistrant extends dw.object.ExtensibleObject {
    /**
     * The email address of the registrant or null.
     */
    email: string

    /**
     * The first name of the registrant or null.
     */
    firstName: string

    /**
     * The last name of the registrant or null.
     */
    lastName: string

    /**
     * The role of the registrant or null (for example "bride").
     */
    role: string

    /**
     * Returns the email address of the registrant or null.
     */
    getEmail(): string

    /**
     * Returns the first name of the registrant or null.
     */
    getFirstName(): string

    /**
     * Returns the last name of the registrant or null.
     */
    getLastName(): string

    /**
     * Returns the role of the registrant or null.
     */
    getRole(): string

    /**
     * Sets the email address of the registrant.
     * @param email the email address to set
     */
    setEmail(email: string): void

    /**
     * Sets the first name of the registrant.
     * @param firstName the first name to set
     */
    setFirstName(firstName: string): void

    /**
     * Sets the last name of the registrant.
     * @param lastName the last name to set
     */
    setLastName(lastName: string): void

    /**
     * Sets the role of the registrant.
     * @param role the role name to set
     */
    setRole(role: string): void

}
```
