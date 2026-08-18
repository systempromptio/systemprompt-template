# dw.object.CustomObjectMgr

## Overview
Manager for creating, retrieving, querying and removing custom objects.

## Description
Provides static methods to create custom objects, describe types, retrieve single instances, iterate all instances, query, and remove instances.

```ts
declare class CustomObjectMgr  {
    /** Create a custom object of `type` with string key. */
    static createCustomObject(type: string, keyValue: string): CustomObject

    /** Create a custom object of `type` with numeric key. */
    static createCustomObject(type: string, keyValue: number): CustomObject

    /** Returns metadata for given type. */
    static describe(type: string): ObjectTypeDefinition

    /** Returns all custom objects of the given type as a SeekableIterator. */
    static getAllCustomObjects(type: string): SeekableIterator

    /** Returns a custom object by type and string key or null. */
    static getCustomObject(type: string, keyValue: string): CustomObject | null

    /** Returns a custom object by type and numeric key or null. */
    static getCustomObject(type: string, keyValue: number): CustomObject | null

    /** Searches for a single custom object matching the query string and args. */
    static queryCustomObject(type: string, queryString: string, ...args: any[]): CustomObject

    /** Searches for custom objects using a query string and returns a SeekableIterator. */
    static queryCustomObjects(type: string, queryString: string, sortString?: string, ...args: any[]): SeekableIterator

    /** Searches for custom objects using a map of attributes and optional sort string. */
    static queryCustomObjects(type: string, queryAttributes: Map, sortString?: string): SeekableIterator

    /** Removes the specified custom object. */
    static remove(object: CustomObject): void
}
```
