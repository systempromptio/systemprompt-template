 # dw.order.PaymentMgr

 ## Overview
 Manager for payment methods and processors; helps lookup and registration of payment methods.

 ## Description
 Manager for payment methods and related utilities used by the order system.

 ```ts
 declare class PaymentMgr  {
 	/** Returns a PaymentMethod by ID. */
 	static getPaymentMethod(id: string): dw.order.PaymentMethod

 	/** Returns all available payment methods for the current site. */
 	static getPaymentMethods(): dw.util.Collection

 	/** Registers a payment method with the system. */
 	static registerPaymentMethod(method: dw.order.PaymentMethod): void
 }
 ```
