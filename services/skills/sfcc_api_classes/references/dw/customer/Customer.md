# dw.customer.Customer

## Overview
Represents a storefront customer with profile, credentials, addresses, order history, groups, and external profiles.

## Description
Main customer object exposing read-only state (IDs, profile, flags) and helper methods to access related objects (address book, order history, product lists, external profiles). Includes methods to query membership in customer groups and to manage external profiles and notes.

```ts
declare class Customer  {
    /** Read-only: collection of customer groups this customer is a member of. */
    readonly customerGroups: Collection<CustomerGroup>

    /** Read-only: whether customer is externally authenticated (OAuth). */
    readonly externallyAuthenticated: boolean

    /** Read-only: collection of externalProfiles attached to the customer. */
    readonly externalProfiles: Collection<ExternalProfile>

    /** Read-only: Global Party ID if present. */
    readonly globalPartyID: string

    /** Read-only: system-generated unique customer ID. */
    readonly ID: string

    /** Read-only: a note attached to the customer (nullable). */
    readonly note: string

    /** Read-only: customer order history. */
    readonly orderHistory: OrderHistory

    /** Read-only: customer profile. */
    readonly profile: Profile

    /** Read-only: whether customer is registered (has profile). */
    readonly registered: boolean

    /** Creates and attaches an ExternalProfile for the given provider and external ID. */
    createExternalProfile(authenticationProviderId: string, externalId: string): ExternalProfile

    /** Returns the active data for this customer. */
    getActiveData(): CustomerActiveData

    /** Returns address book for this customer's profile, or null if no profile. */
    getAddressBook(): AddressBook

    /** Returns Salesforce CDP data for this customer. */
    getCDPData(): CustomerCDPData

    /** Returns collection of CustomerGroups for this customer. */
    getCustomerGroups(): Collection<CustomerGroup>

    /** Convenience to find an ExternalProfile by provider and externalId. */
    getExternalProfile(authenticationProviderId: string, externalId: string): ExternalProfile

    /** Returns all external profiles for this customer. */
    getExternalProfiles(): Collection<ExternalProfile>

    /** Returns the Global Party ID if present. */
    getGlobalPartyID(): string

    /** Returns the system-generated customer ID. */
    getID(): string

    /** Returns the customer note or null. */
    getNote(): string

    /** Returns the customer's order history. */
    getOrderHistory(): OrderHistory

    /** Returns product lists of the specified type. */
    getProductLists(type: number): Collection<ProductList>

    /** Returns the customer profile. */
    getProfile(): Profile

    /** True if this customer is anonymous. */
    isAnonymous(): boolean

    /** True if this customer is authenticated for the current session. */
    isAuthenticated(): boolean

    /** True if externally authenticated (OAuth). */
    isExternallyAuthenticated(): boolean

    /** Returns true if customer is member of any of the provided group IDs. */
    isMemberOfAnyCustomerGroup(...groupIDs: string[]): boolean

    /** Returns true if customer is member of the given CustomerGroup. */
    isMemberOfCustomerGroup(group: CustomerGroup): boolean

    /** Returns true if customer is member of customer group identified by groupID. */
    isMemberOfCustomerGroup(groupID: string): boolean

    /** Returns true if customer is member of all provided group IDs. */
    isMemberOfCustomerGroups(...groupIDs: string[]): boolean

    /** True if customer is registered (has profile). */
    isRegistered(): boolean

    /** Removes an attached external profile. */
    removeExternalProfile(externalProfile: ExternalProfile): void

    /** Sets a free-text note for the customer. */
    setNote(aValue: string): void
}
```
