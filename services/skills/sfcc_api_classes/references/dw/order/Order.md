# dw.order.Order

## Overview
Represents an order and provides getters, creators and status constants.

## Description
The Order class represents an order. Use OrderMgr to retrieve orders.

```ts
declare class Order extends LineItemCtnr {
	/** CONFIRMATION_STATUS_CONFIRMED: 2 */
	static CONFIRMATION_STATUS_CONFIRMED: 2

	/** CONFIRMATION_STATUS_NOTCONFIRMED: 0 */
	static CONFIRMATION_STATUS_NOTCONFIRMED: 0

	/** many other numeric/string constants (ENCRYPTION_ALGORITHM..., EXPORT_STATUS_..., ORDER_STATUS_..., PAYMENT_STATUS_..., SHIPPING_STATUS_...) */

	/** affiliate partner ID */
	affiliatePartnerID: string

	/** affiliate partner name */
	affiliatePartnerName: string

	/** Captured amount (read-only) */
	/** @readonly */
	capturedAmount: Money

	/** Confirmation status */
	confirmationStatus: EnumValue

	/** Creator name (read-only) */
	/** @readonly */
	createdBy: string

	/** Current order (read-only) */
	/** @readonly */
	currentOrder: Order

	/** Current order number (read-only) */
	/** @readonly */
	currentOrderNo: string

	/** Customer locale ID (read-only) */
	/** @readonly */
	customerLocaleID: string

	/** ...additional properties omitted for brevity but must be present in the source HTML. */

	/** Creates an appeasement for the order. */
	createAppeasement(appeasementNumber?: string): Appeasement

	/** Create return case. */
	createReturnCase(returnCaseNumber?: string, isRMA?: boolean): ReturnCase

	/** Create service item wrapper. */
	createServiceItem(ID: string, status: string): OrderItem

	/** Create shipping order. */
	createShippingOrder(shippingOrderNumber?: string): ShippingOrder

	/** Many getters such as getAffiliatePartnerID, getAppeasements, getCapturedAmount, getOrderExportXML etc. */

}
```
