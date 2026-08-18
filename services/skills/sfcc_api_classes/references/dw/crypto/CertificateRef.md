# dw.crypto.CertificateRef

## Overview
Reference to a certificate or public key stored in Business Manager. Handles sensitive security-related data.

## Description
Used as a lightweight reference to a certificate or public key by alias. No validation of the alias is performed until the reference is resolved. This class handles sensitive security-related data; follow applicable PCI DSS requirements when using it.

## All Known Subclasses
X509Certificate

```ts
declare class CertificateRef  {
    /**
     * Creates a CertificateRef from the passed alias as a reference to a certificate in Business Manager.
     * No check is made whether the alias is valid until the reference is resolved.
     * @param alias An alias that should refer to a certificate in the keystore.
     */
    constructor(alias: string)

    /**
     * Returns the string representation of this CertificateRef.
     */
    toString(): string
}
```
