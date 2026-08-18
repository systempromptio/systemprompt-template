# dw.customer.CustomerMgr

## Overview
Manager class providing profile and authentication operations for customers (search, login, logout, removal, etc.).

## Description
CustomerMgr exposes many static helper methods to search, query, authenticate, login/logout customers, and manage customer profiles in batch or interactive contexts. Several legacy query methods are deprecated in favor of newer search APIs.

```ts
declare class CustomerMgr  {
    /**
     * Execute a function for each matching profile (intended for batch jobs).
     * @param processFunction - Function(profile)
     * @param queryString - query expression
     * @param args - optional arguments for the query
     */
    static processProfiles(processFunction: Function, queryString: string, ...args: any[]): void

    /**
     * Search for a single profile using a query string (deprecated in favor of searchProfile variants).
     */
    static queryProfile(queryString: string, ...args: any[]): Profile

    /**
     * Query multiple profiles (deprecated).
     */
    static queryProfiles(queryString: string, sortString: string, ...args: any[]): dw.util.SeekableIterator

    /**
     * Remove (delete) a registered customer and related profile data. Customer must be logged in.
     * @param customer - Customer to remove
     */
    static removeCustomer(customer: Customer): void

    /**
     * Remove asynchronous tracking data for a customer.
     * @param customer - Customer whose tracking data will be removed
     */
    static removeCustomerTrackingData(customer: Customer): void

    /**
     * Search for a profile using the modern search API.
     */
    static searchProfile(queryString: string, ...args: any[]): Profile

    /**
     * Search multiple profiles using modern API.
     */
    static searchProfiles(queryString: string, sortString: string, ...args: any[]): dw.util.SeekableIterator

    /**
     * Log in previously authenticated customer. Returns Customer or null.
     */
    static loginCustomer(authStatus: AuthenticationStatus, rememberMe: boolean): Customer

    /**
     * Log in an externally authenticated customer by provider id and external id.
     */
    static loginExternallyAuthenticatedCustomer(authenticationProviderId: string, externalId: string, rememberMe: boolean): Customer

    /**
     * Logout the currently authenticated customer. Returns new identity.
     */
    static logoutCustomer(rememberMe: boolean): Customer
}
```
