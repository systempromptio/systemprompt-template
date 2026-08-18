# dw.order.PaymentProcessor

## Overview
A container for payment processor configuration values (merchant IDs, URLs, etc.).

## Description
A PaymentProcessor represents an entity that processes payments of one or more types. System processors provide preference values maintained in Business Manager; merchants may also define custom processors whose preferences are exposed via `getPreferenceValue(String)`.

```ts
declare class PaymentProcessor extends dw.object.PersistentObject {
    /**
     * The 'ID' of this processor.
     * @readonly
     */
    ID: string

    /**
     * Returns the 'ID' of this processor.
     */
    getID(): string

    /**
     * Returns the value of the specified preference for this payment processor.
     * @param name preference name
     */
    getPreferenceValue(name: string): unknown
}
```
