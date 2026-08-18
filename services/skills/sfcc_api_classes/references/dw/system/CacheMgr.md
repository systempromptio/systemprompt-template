# dw.system.CacheMgr

## Overview
Utility manager for caching operations and cache instance access.

## Description
Exposes methods to get, put, and clear caches by name. Used to improve performance by storing computed values.

```ts
declare class CacheMgr {
    /**
     * Returns a cache instance by ID.
     * @param id string
     */
    static getCache(id: string): any

    /**
     * Clears the named cache.
     * @param id string
     */
    static clearCache(id: string): void
}
```
