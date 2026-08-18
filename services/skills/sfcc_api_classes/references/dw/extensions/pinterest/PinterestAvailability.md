# dw.extensions.pinterest.PinterestAvailability

## Overview
Represents a row in the Pinterest availability feed export file (product availability and ID).

## Description
A simple container for Pinterest product availability and ID used when exporting product availability to Pinterest.

```ts
declare class PinterestAvailability  {
  /** The availability string (e.g. AVAILABILITY_IN_STOCK or AVAILABILITY_OUT_OF_STOCK). */
  availability: string

  /** The ID of the Pinterest product (same as the Demandware product ID). */
  ID: string

  /** Returns the availability of the Pinterest product. */
  getAvailability(): string

  /** Returns the ID of the Pinterest product. */
  getID(): string

  /** Sets the availability of the Pinterest product. */
  setAvailability(availability: string): void
}
```

All Known Subclasses

None
