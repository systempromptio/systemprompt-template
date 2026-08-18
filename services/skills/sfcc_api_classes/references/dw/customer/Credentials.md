# dw.customer.Credentials

## Overview
Represents authentication-related data for a customer: login, password state, lock/enabled status, and methods to manage credentials.

## Description
Provides read-only properties and mutating methods to inspect and update a customer's authentication details (login, password, external IDs, provider ID, password question/answer). Several methods dealing with external authentication are deprecated in favor of `ExternalProfile`.

```ts
declare class Credentials  {
    /** Returns a unique token valid for 30 minutes to reset the customer's password. */
    createResetPasswordToken(): string

    /** Deprecated: Returns authentication provider ID for externally-authenticated customers. */
    getAuthenticationProviderID(): string

    /** Returns whether the customer is enabled (can log in). */
    getEnabledFlag(): boolean

    /** Deprecated: Returns external ID assigned by the authentication provider. */
    getExternalID(): string

    /** Returns the customer's login (unique). */
    getLogin(): string

    /** Returns the password answer used for recovery. */
    getPasswordAnswer(): string

    /** Returns the password question used for recovery. */
    getPasswordQuestion(): string

    /** Returns remaining attempts before lockout (may be negative or 0 as described). */
    getRemainingLoginAttempts(): number

    /** Returns whether the customer account is enabled. */
    isEnabled(): boolean

    /** Returns whether the customer is locked out due to failed attempts. */
    isLocked(): boolean

    /** Returns whether a password has been set for this customer. */
    isPasswordSet(): boolean

    /** Deprecated: set authentication provider ID for external auth. */
    setAuthenticationProviderID(authenticationProviderID: string): void

    /** Sets whether the customer is enabled. */
    setEnabledFlag(enabledFlag: boolean): void

    /** Deprecated: sets external ID at provider. */
    setExternalID(externalID: string): void

    /** Deprecated: sets login directly (do not use). */
    setLogin(login: string): void

    /** Sets login and re-encrypts password; returns true if successful. */
    setLogin(newLogin: string, currentPassword: string): boolean

    /** Sets the customer's password (may return Status). */
    setPassword(newPassword: string, oldPassword: string, verifyOldPassword: boolean): Status

    /** Sets the password answer. */
    setPasswordAnswer(answer: string): void

    /** Sets the password question. */
    setPasswordQuestion(question: string): void

    /** Sets password using a reset token; returns Status. */
    setPasswordWithToken(token: string, newPassword: string): Status

    /** Read-only: external ID (deprecated). */
    readonly externalID: string

    /** Read-only: whether the account is locked. */
    readonly locked: boolean

    /** Read-only: login value. */
    readonly login: string

    /** Read-only: password answer. */
    readonly passwordAnswer: string

    /** Read-only: password question. */
    readonly passwordQuestion: string

    /** Read-only: whether a password is set. */
    readonly passwordSet: boolean

    /** Read-only: number of remaining login attempts before lockout. */
    readonly remainingLoginAttempts: number
}
```
