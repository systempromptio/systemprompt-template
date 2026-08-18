 # dw.catalog.ProductOptionValue

 ## Overview
 Represents a single value of a product option with localized display name and description, an ID, and a product ID modifier for SKU construction.

 ## Description
 Read-only access to localized description and display value, value identifier, and product ID modifier used to form variant SKUs.

 ```ts
 declare class ProductOptionValue extends PersistentObject {
    /** The product option value's description in the current locale. */
    readonly description: string

    /** The product option value's display name in the current locale. */
    readonly displayValue: string

    /** The product option value's ID. */
    readonly ID: string

    /** Product ID modifier used to build SKU for the actual product. */
    readonly productIDModifier: string

    /** Returns the product option value's description in the current locale, or null if not found. */
    getDescription(): string

    /** Returns the product option value's display name in the current locale, or null if not found. */
    getDisplayValue(): string

    /** Returns the product option value's ID. */
    getID(): string

    /** Returns the product option value's product ID modifier used to build the SKU. */
    getProductIDModifier(): string
 }
 ```
