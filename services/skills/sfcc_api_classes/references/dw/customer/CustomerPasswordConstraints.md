# dw.customer.CustomerPasswordConstraints

## Overview
Provides read-only access to password policy constraints for customers (min length, special chars, and rules).

## Description
Use to inspect enforced password constraints (letters, mixed case, numbers, minimum length, minimum special characters).

```ts
declare class CustomerPasswordConstraints  {
    /**
     * True if letters are required (read-only).
     */
    forceLetters: boolean

    /**
     * True if mixed case is required (read-only).
     */
    forceMixedCase: boolean

    /**
     * True if numbers are required (read-only).
     */
    forceNumbers: boolean

    /**
     * Minimum password length (read-only).
     */
    minLength: number

    /**
     * Minimum special characters required (read-only).
     */
    minSpecialChars: number

    /**
     * Returns the minimum length.
     */
    static getMinLength(): number

    /**
     * Returns minimum number of special characters.
     */
    static getMinSpecialChars(): number

    /**
     * Returns true if letters are required.
     */
    static isForceLetters(): boolean

    /**
     * Returns true if mixed case is required.
     */
    static isForceMixedCase(): boolean

    /**
     * Returns true if numbers are required.
     */
    static isForceNumbers(): boolean
}
```
