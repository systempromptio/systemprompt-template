# dw.crypto.KeyRef

## Overview
Reference to a private key in the keystore managed in Business Manager. Handles sensitive security-related data.

## Description
Represents a reference to a private key identified by an alias. Validation that the alias exists is deferred until the reference is used. A deprecated constructor accepted a password; prefer the single-argument constructor.

```ts
declare class KeyRef  {
    /**
     * Creates a KeyRef from the passed alias.
     * @param alias Alias referring to a key in the keystore.
     */
    constructor(alias: string)

    /**
     * Deprecated. Creates a KeyRef with a password. Use KeyRef(String) instead.
     * @param alias Alias referring to a key in the keystore.
     * @param password Password used to retrieve the key.
     */
    constructor(alias: string, password: string)

    /**
     * Returns the string representation of this KeyRef.
     */
    toString(): string
}
```
