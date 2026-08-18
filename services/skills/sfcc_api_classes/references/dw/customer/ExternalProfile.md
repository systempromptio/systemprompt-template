# dw.customer.ExternalProfile

## Overview
Represents credentials and authentication details for a customer authenticated by an external OAuth2 provider.

## Description
Holds external authentication metadata: provider ID, external user ID, email and last login timestamp. Intended for customers authenticated via OAuth2 providers (e.g., Google). Contains sensitive security-related data.

```ts
declare class ExternalProfile {
  /** Read-only: authentication provider identifier (e.g. configured provider ID). */
  authenticationProviderID: string

  /** Read-only: the `Customer` that owns this external profile. */
  customer: Customer

  /** The customer's email address. */
  email: string

  /** Read-only: the external identifier assigned by the authentication provider. */
  externalID: string

  /** Read-only: last login time through the external provider. */
  lastLoginTime: Date

  /** Returns the authentication provider ID. */
  getAuthenticationProviderID(): string

  /** Returns the related `Customer` object. */
  getCustomer(): Customer

  /** Returns the customer's email. */
  getEmail(): string

  /** Returns the external ID assigned by the provider. */
  getExternalID(): string

  /** Returns the last login time via the external provider. */
  getLastLoginTime(): Date

  /** Sets the customer's email address. @param email the new email */
  setEmail(email: string): void
}
```
