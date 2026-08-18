# dw.object.SystemObjectMgr

## Overview
Manager for querying system objects (and their metadata) using the Commerce Cloud query language.

## Description
Provides methods to describe system object types and query system objects (e.g., GiftCertificate, Store, ProductList). Use caution: queries may access sensitive Profile and Order data.

```ts
declare class SystemObjectMgr  {
    /** Returns the object type definition for the given system object type. */
    static describe(type: string): dw.object.ObjectTypeDefinition | null

    /** Returns all system objects of a specific type as a SeekableIterator. */
    static getAllSystemObjects(type: string): dw.util.SeekableIterator

    /** Searches for a single system object instance using a query string. */
    static querySystemObject(type: string, queryString: string, ...args: any[]): dw.object.PersistentObject | null

    /** Searches for system object instances using a query string and sort expression. */
    static querySystemObjects(type: string, queryString: string, sortString?: string, ...args: any[]): dw.util.SeekableIterator

    /** Searches for system objects using query attributes map. */
    static querySystemObjects(type: string, queryAttributes: dw.util.Map, sortString?: string): dw.util.SeekableIterator
}
```
