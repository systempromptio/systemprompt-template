 # dw.order.PaymentMethod

 ## Overview
 Represents a payment method available to customers and orders.

 ## Description
 Represents a payment method. Provides metadata and helper methods to interact with payment providers and determine display attributes.

 ```ts
 declare class PaymentMethod  {
 	/** The identifier for this payment method. */
 	id: string

 	/** The display name for this payment method. */
 	name: string

 	/** The payment processor ID used by this method. */
 	paymentProcessorId: string

 	/** True if the method requires billing address. */
 	requiresBillingAddress: boolean

 	/** True if the method supports tokenization. */
 	supportsTokenization: boolean

 	/** Returns whether this method is enabled for the given site or context. */
 	isEnabled(siteId?: string): boolean

 	/** Returns a localized display label for UI. */
 	getDisplayLabel(locale?: string): string
 }
 ```
