# dw.order.hooks.ShippingOrderHooks

## Overview
Interface for script hooks around shipping order lifecycle.

## Description
Represents all script hooks that can be registered around shipping order lifecycle. Contains extension points (hook names) and functions called by each extension point. Hook functions must be defined inside a JavaScript source, exported, and registered via `hooks.json` in a site cartridge's `package.json`.

```ts
declare class ShippingOrderHooks  {
	/**
	 * Extension point name for after status change.
	 */
	static extensionPointAfterStatusChange: 'dw.order.shippingorder.afterStatusChange'

	/**
	 * Extension point name for changing status.
	 */
	static extensionPointChangeStatus: 'dw.order.shippingorder.changeStatus'

	/**
	 * Extension point name for creating shipping orders.
	 */
	static extensionPointCreateShippingOrders: 'dw.order.shippingorder.createShippingOrders'

	/**
	 * Extension point name for notifying status change.
	 */
	static extensionPointNotifyStatusChange: 'dw.order.shippingorder.notifyStatusChange'

	/**
	 * Extension point name for preparing shipping order creation.
	 */
	static extensionPointPrepareCreateShippingOrders: 'dw.order.shippingorder.prepareCreateShippingOrders'

	/**
	 * Extension point name for resolving shipping order.
	 */
	static extensionPointResolveShippingOrder: 'dw.order.shippingorder.resolveShippingOrder'

	/**
	 * Extension point name for setting shipping order cancelled.
	 */
	static extensionPointShippingOrderCancelled: 'dw.order.shippingorder.setShippingOrderCancelled'

	/**
	 * Extension point name for setting shipping order shipped.
	 */
	static extensionPointShippingOrderShipped: 'dw.order.shippingorder.setShippingOrderShipped'

	/**
	 * Extension point name for setting shipping order to warehouse.
	 */
	static extensionPointShippingOrderWarehouse: 'dw.order.shippingorder.setShippingOrderWarehouse'

	/**
	 * Extension point name for updating shipping order item.
	 */
	static extensionPointUpdateShippingOrderItem: 'dw.order.shippingorder.updateShippingOrderItem'

	/**
	 * Called after status change, runs inside transaction.
	 * @param shippingOrder - the shipping order to be updated
	 */
	afterStatusChange(shippingOrder: ShippingOrder): Status

	/**
	 * Changes the status of a shipping order.
	 * @param shippingOrder - the shipping order to be updated
	 * @param updateData - the input data
	 */
	changeStatus(shippingOrder: ShippingOrder, updateData: unknown): Status

	/**
	 * Creates shipping orders for an order.
	 * @param order - the order to create shipping orders for
	 */
	createShippingOrders(order: Order): Status

	/**
	 * Notifies of status change, runs outside transaction.
	 * @param shippingOrder - the shipping order to be updated
	 */
	notifyStatusChange(shippingOrder: ShippingOrder): Status

	/**
	 * Prepares for shipping order creation.
	 * @param order - the order to create shipping orders for
	 */
	prepareCreateShippingOrders(order: Order): Status

	/**
	 * Resolves the shipping order.
	 * @param updateData - the input data
	 */
	resolveShippingOrder(updateData: unknown): ShippingOrder

	/**
	 * Sets shipping order status to cancelled (optional).
	 * @param updateData - the input data
	 */
	setShippingOrderCancelled(updateData: unknown): Order

	/**
	 * Sets shipping order status to shipped (optional).
	 * @param updateData - the input data
	 */
	setShippingOrderShipped(updateData: unknown): Order

	/**
	 * Sets shipping order status to warehouse (optional).
	 * @param updateData - the input data
	 */
	setShippingOrderWarehouse(updateData: unknown): Order

	/**
	 * Updates the status of a shipping order item.
	 * @param shippingOrder - the shipping order
	 * @param updateItem - the input data
	 */
	updateShippingOrderItem(shippingOrder: ShippingOrder, updateItem: unknown): Status
}
```
