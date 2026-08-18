# dw.object.ObjectAttributeValueDefinition

## Overview
Represents the value definition associated with an object attribute.

## Description
Encapsulates an allowed attribute value with both the stored value and a localized display name for UI presentation.

```ts
declare class ObjectAttributeValueDefinition  {
    /** Display name for UI. */
    displayValue: string // (Read Only)

    /** Actual value for the attribute. */
    value: any // (Read Only)

    /** Returns the display name for this value. */
    getDisplayValue(): string

    /** Returns the actual value. */
    getValue(): any
}
```
