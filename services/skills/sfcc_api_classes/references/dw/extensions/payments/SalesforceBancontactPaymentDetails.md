# dw.extensions.payments.SalesforceBancontactPaymentDetails

## Overview
Details for a Salesforce Payments BANCONTACT payment: bank name and last 4 digits.

## Description
Payment details object for SalesforcePaymentMethod.TYPE_BANCONTACT. Exposes read-only properties `bankName` and `last4` and corresponding getters.

##
```ts
declare class SalesforceBancontactPaymentDetails extends SalesforcePaymentDetails {
	/** Read-only bank name or null if unknown. */
	readonly bankName: string
	/** Read-only last 4 digits of the account number or null if unknown. */
	readonly last4: string

	getBankName(): string
	getLast4(): string
}
```
