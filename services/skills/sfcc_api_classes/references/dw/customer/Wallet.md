# dw.customer.Wallet

## Overview
Represents a customer's set of payment instruments (wallet). Provides creation, listing and removal of payment instruments tied to a customer.

## Description
Wallet exposes methods to create a new empty payment instrument for a given payment method, retrieve all or filtered payment instruments, and remove an associated payment instrument. Methods may throw when arguments are null or invalid.

```ts
declare class Wallet  {
    /**
     * Read-only collection of payment instruments associated with the customer.
     */
    paymentInstruments: dw.util.Collection

    /**
     * Creates a new, empty payment instrument object for the given payment method id.
     * @param paymentMethodId id of the payment method
     * @throws NullArgumentException when paymentMethodId is null
     */
    createPaymentInstrument(paymentMethodId: string): dw.customer.CustomerPaymentInstrument

    /**
     * Returns a collection of all payment instruments associated with the customer.
     */
    getPaymentInstruments(): dw.util.Collection

    /**
     * Returns a collection of payment instruments filtered by `paymentMethodID`.
     * Passing null returns all instruments.
     * @param paymentMethodID id to filter by
     */
    getPaymentInstruments(paymentMethodID: string): dw.util.Collection

    /**
     * Removes the given payment instrument from the customer.
     * @param instrument the payment instrument to remove
     * @throws NullArgumentException when instrument is null
     * @throws IllegalArgumentException when instrument belongs to another customer
     */
    removePaymentInstrument(instrument: dw.customer.CustomerPaymentInstrument): void
}
```
