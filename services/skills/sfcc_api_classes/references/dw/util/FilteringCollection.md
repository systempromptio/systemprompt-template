# dw.util.FilteringCollection

## Overview
An extension of Collection that provides methods to filter, sort, and transform elements.

## Description
FilteringCollection is an extension of Collection which provides possibilities to:

- **filter** the elements to return a new FilteringCollection with a filtered set of elements
- **sort** the elements to return a new FilteringCollection with a defined sort order
- **transform** the elements to return a new FilteringCollection containing related elements
- **provide a map** of the elements against a predefined key

**Usage** - In the current version each FilteringCollection provides a set of predefined **qualifier** constants which can be passed into the select(Object) method used to **filter** the elements. Generally **qualifiers** have the prefix `QUALIFIER_`. A second method sort(Object) is used to create a new instance with a different element ordering, which takes an **orderBy** constant. Generally **orderBys** have the prefix `ORDERBY_`: examples are ShippingOrder.ORDERBY_ITEMID, ShippingOrder.ORDERBY_ITEMPOSITION, and ORDERBY_REVERSE can be used to provide a FilteringCollection with the reverse ordering.

An example with method ShippingOrder.getItems():

```js
var allItems     : FilteringCollection = shippingOrder.items;
var productItems : FilteringCollection = allItems.select(ShippingOrder.QUALIFIER_PRODUCTITEMS);
var serviceItems : FilteringCollection = allItems.select(ShippingOrder.QUALIFIER_SERVICEITEMS);
var byPosition   : FilteringCollection = productItems.sort(ShippingOrder.ORDERBY_ITEMPOSITION);
var revByPosition: FilteringCollection = byPosition.sort(FilteringCollection.ORDERBY_REVERSE);
var mapByItemID  : Map = allItems.asMap();
```

## Hierarchy
```
Object
  dw.util.Collection
    dw.util.FilteringCollection
```

## TypeScript Declaration

```ts
declare class FilteringCollection extends Collection {
	/**
	 * Pass this orderBy with the sort(Object) method to obtain a new FilteringCollection with the reversed sort order. Only use on a FilteringCollection which has been previously sorted.
	 */
	static ORDERBY_REVERSE: Object

	/**
	 * Returns a Map containing the elements of this FilteringCollection against a predefined key. The key used is documented in the method returning the FilteringCollection and is typically the ItemID assigned to an element in the collection.
	 * @returns a Map containing the elements of this FilteringCollection against a predefined key
	 */
	asMap(): Map

	/**
	 * Select a new FilteringCollection instance by passing a predefined qualifier as an argument to this method. See FilteringCollection.
	 * @param qualifier - possible qualifiers are documented in the method returning the FilteringCollection
	 * @returns a new FilteringCollection instance
	 */
	select(qualifier: Object): FilteringCollection

	/**
	 * Select a new FilteringCollection instance by passing a predefined orderBy as an argument to this method. See FilteringCollection.
	 * @param orderBy - possible orderBys are documented in the method returning the FilteringCollection
	 * @returns a new FilteringCollection instance
	 */
	sort(orderBy: Object): FilteringCollection
}
```
