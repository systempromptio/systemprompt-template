
# dw.util.LinkedHashMap

## Overview
HashMap implementation that guarantees iteration order according to the put-order of elements.

## Description
This class implements a HashMap, which guarantees a iteration order according the put-order of the elements in the map.

```
Object
  dw.util.Map
    dw.util.LinkedHashMap
```

```ts
declare class LinkedHashMap extends Map {
	/**
	 * Constructs a new LinkedHashMap.
	 */
	constructor()

	/**
	 * Returns a shallow copy of this map.
	 * @returns a shallow copy of this map
	 */
	clone(): LinkedHashMap
}
```
