# dw.customer.EncryptedObject

## Overview
Base class for objects that contain encrypted attributes (for example, credit card data).

## Description
Defines an API base class for classes holding encrypted customer-related attributes. Handles sensitive financial and cardholder data; follow PCI DSS requirements when accessing these objects.

## All Known Subclasses
CustomerPaymentInstrument, OrderPaymentInstrument, PaymentInstrument, Profile, ServiceCredential

```ts
declare class EncryptedObject extends ExtensibleObject {
  /** Class handles encrypted attributes like credit cards. */
  constructor(): void
}
```
