# dw.system.Cache

## Overview
Provides a custom cache for storing data across multiple requests with configurable eviction strategies.

## Description
The Cache class represents a custom cache that stores data over multiple requests. Each cartridge can define its own caches for different business requirements. To limit visibility of cache entries by scope (site, catalog, or external system), include the scope reference when constructing the key.

Do not build cache keys using personal user data, since keys might be visible in log messages.

There is never a guarantee that a stored object can be retrieved from the cache. Storage is limited and clearing or invalidation might occur at any time. To maintain size limits, the cache evicts entries that are less likely to be used again. Cache entries aren't synchronized between different application servers.

The cache returns immutable copies of the original objects put into the cache. Lists are converted to arrays during this process. Only JavaScript primitive values and tree-like object structures can be stored as entries. Object structures can consist of arrays, lists, and basic JavaScript objects. Script API classes are not supported, except List and its subclasses. `null` can be stored as a value. `undefined` can't be stored.

```
Object
  dw.system.Cache
```

```ts
declare class Cache {
	/**
	 * Returns the value associated with key in this cache, or invokes the loader function to generate the entry if there is no entry found.
	 * The generated entry is stored for future retrieval. If the loader function returns undefined, this value is not stored in the cache.
	 * @param key - The cache key
	 * @param loader - The loader function that is called if no value is stored in the cache
	 * @returns The value found in the cache or the value returned from the loader function call
	 */
	get(key: string, loader: Function): Object

	/**
	 * Returns the value associated with key in this cache.
	 * If there is no entry in the cache then undefined is returned.
	 * @param key - The cache key
	 * @returns The stored value or undefined if no value is found in the cache
	 */
	get(key: string): Object

	/**
	 * Removes the cache entry for key (if one exists) manually before the cache's eviction strategy goes into effect.
	 * @param key - The cache key
	 */
	invalidate(key: string): void

	/**
	 * Stores the specified entry directly into the cache, replacing any previously cached entry for key if one exists.
	 * Storing undefined as value has the same effect as calling invalidate(String) for that key.
	 * @param key - The cache key
	 * @param value - The value to be stored in the cache
	 */
	put(key: string, value: Object): void
}
```
