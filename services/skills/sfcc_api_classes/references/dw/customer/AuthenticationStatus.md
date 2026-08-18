# dw.customer.AuthenticationStatus

## Overview
Represents the outcome of an authentication attempt: status code, whether authenticated, and the matching customer.

## Description
Encapsulates the result of a customer authentication process. Contains constants for common authentication results and read-only properties for the resolved `customer`, `status` code, and a boolean `authenticated` flag.

```ts
declare class AuthenticationStatus  {
    /** Authentication was successful. */
    static AUTH_OK: 'AUTH_OK'

    /** Customer found but disabled; password not verified. */
    static ERROR_CUSTOMER_DISABLED: 'ERROR_CUSTOMER_DISABLED'

    /** Customer found but locked (too many failed attempts); password was verified. */
    static ERROR_CUSTOMER_LOCKED: 'ERROR_CUSTOMER_LOCKED'

    /** Customer could not be found. */
    static ERROR_CUSTOMER_NOT_FOUND: 'ERROR_CUSTOMER_NOT_FOUND'

    /** Password matches but is expired. */
    static ERROR_PASSWORD_EXPIRED: 'ERROR_PASSWORD_EXPIRED'

    /** Provided password does not match. */
    static ERROR_PASSWORD_MISMATCH: 'ERROR_PASSWORD_MISMATCH'

    /** Any other error. */
    static ERROR_UNKNOWN: 'ERROR_UNKNOWN'

    /** Read-only: whether authentication succeeded. */
    readonly authenticated: boolean

    /** Read-only: the Customer corresponding to the login (not logged in). */
    readonly customer: Customer

    /** Read-only: status code string from the constants above. */
    readonly status: string

    /** Returns the associated Customer. */
    getCustomer(): Customer

    /** Returns the status code string. */
    getStatus(): string

    /** Returns whether authentication succeeded. */
    isAuthenticated(): boolean
}
```
