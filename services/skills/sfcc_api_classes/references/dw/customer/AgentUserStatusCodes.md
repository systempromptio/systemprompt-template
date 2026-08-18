# dw.customer.AgentUserStatusCodes

## Overview
Constants representing status codes used with `Status` to indicate the result of an agent-user login attempt.

## Description
Contains string constants for success and various failure states encountered during agent user login (invalid credentials, locked account, insecure connection, missing permissions, etc.). Use these with `dw.system.Status` to communicate login outcomes.

## All Known Subclasses
- dw.system.AgentUserStatusCodes

```ts
declare class AgentUserStatusCodes  {
    /** Indicates that the agent user is not available. */
    static AGENT_USER_NOT_AVAILABLE: 'AGENT_USER_NOT_AVAILABLE'

    /** Indicates that the agent user is not logged in. */
    static AGENT_USER_NOT_LOGGED_IN: 'AGENT_USER_NOT_LOGGED_IN'

    /** Indicates that the given agent user login or password was wrong. */
    static CREDENTIALS_INVALID: 'CREDENTIALS_INVALID'

    /** Indicates that the customer is disabled. */
    static CUSTOMER_DISABLED: 'CUSTOMER_DISABLED'

    /** Indicates that the customer is not registered or not registered with the current site. */
    static CUSTOMER_UNREGISTERED: 'CUSTOMER_UNREGISTERED'

    /** Indicates that the current connection is not secure while a secure connection is required. */
    static INSECURE_CONNECTION: 'INSECURE_CONNECTION'

    /** Indicates the agent user lacks the required 'Login_Agent' permission. */
    static INSUFFICIENT_PERMISSION: 'INSUFFICIENT_PERMISSION'

    /** Indicates that the agent user login was successful. */
    static LOGIN_SUCCESSFUL: 'LOGIN_SUCCESSFUL'

    /** Indicates the current context is not a storefront request. */
    static NO_STOREFRONT: 'NO_STOREFRONT'

    /** Indicates that the agent user password has expired and must be changed in Business Manager. */
    static PASSWORD_EXPIRED: 'PASSWORD_EXPIRED'

    /** Indicates that the agent user account has been disabled in Business Manager. */
    static USER_DISABLED: 'USER_DISABLED'

    /** Indicates that the agent user account is locked because of too many failed login attempts. */
    static USER_LOCKED: 'USER_LOCKED'

    /**
     * Default constructor.
     */
    constructor()
}
```
