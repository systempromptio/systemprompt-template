# dw.extensions.pinterest.PinterestFeedHooks

## Overview
Hook interface for customizing Pinterest export feeds (transformProduct, transformAvailability).

## Description
Defines extension points for transforming product and availability entries during Pinterest feed exports. Hooks are executed outside transactions and must be provided via site cartridge hook scripts.

```ts
declare class PinterestFeedHooks {
  /** Hook name for transforming availability entries. */
  static extensionPointTransformAvailability: 'dw.extensions.pinterest.feed.transformAvailability'

  /** Hook name for transforming product entries. */
  static extensionPointTransformProduct: 'dw.extensions.pinterest.feed.transformProduct'

  /**
   * Called after default transformation of a Product to PinterestAvailability during availability export.
   * @param product - Demandware product
   * @param pinterestAvailability - PinterestAvailability object
   * @returns Status (non-null Status ends hook execution)
   */
  transformAvailability(product: any, pinterestAvailability: any): dw.system.Status

  /**
   * Called after default transformation of a Product to PinterestProduct during catalog export.
   * @param product - Demandware product
   * @param pinterestProduct - PinterestProduct object
   * @returns Status (non-null Status ends hook execution)
   */
  transformProduct(product: any, pinterestProduct: any): dw.system.Status
}
```

All Known Subclasses

None
