# dw.extensions.payments.SalesforceCardPaymentDetails

## Overview
Details for a Salesforce Payments card payment: brand, last4, and wallet type.

## Description
Payment details object for SalesforcePaymentMethod.TYPE_CARD. Exposes read-only properties `brand`, `last4`, and `walletType` with getters and inherits common behavior from `SalesforcePaymentDetails`.

##
```ts
declare class SalesforceCardPaymentDetails extends SalesforcePaymentDetails {
	/** Read-only card brand or null if unknown. */
	readonly brand: string
	/** Read-only last 4 digits of the card number or null if unknown. */
	readonly last4: string
	/** Read-only wallet type or null if unknown. */
	readonly walletType: string

	getBrand(): string
	getLast4(): string
	getWalletType(): string
}
```
