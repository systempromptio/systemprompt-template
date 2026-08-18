# dw.util.MappingMgr

## Overview
Manages and interfaces with key-value mappings loaded into the system, enabling high-performance lookups independent of the database.

## Description
Used to manage and interface with mappings loaded into the system via the ImportKeyValueMapping job step. Class can be used to retrieve values for known keys, iterate over all keys known in a mapping or list all known mappings. Mappings are read into the system using the ImportKeyValueMapping job step. Generic mapping capability enables you to map keys to values, with the mapping stored in a high-performance data store that is independent of the database. This supports large datasets, with high performance for lookup. An example of using this feature is to map SKUs from a backend system to Commerce Cloud Digital SKUs on-the-fly in Digital script, so that interaction with the backend system is transparent and does not require adding Digital SKUs to the third party system.

```ts
declare class MappingMgr  {
  /**
   * List all known mappings.
   */
  readonly mappingNames: Collection

  /**
   * Returns a map containing value(s) associated to the specified key for the specified mapping.
   * @param mappingName - the mapping name
   * @param key - the key
   * @returns Map containing values for the key
   * @throws IllegalArgumentException if mappingName is unknown
   */
  static get(mappingName: string, key: MappingKey): Map

  /**
   * Gets the first string value of a mapping by name and key. Ordering is determined by the input CSV file.
   * @param mappingName - the mapping name
   * @param key - the key
   * @returns the value if a single value. The first value sequentially if a compound value.
   * @throws IllegalArgumentException if mappingName is unknown
   */
  static getFirst(mappingName: string, key: MappingKey): string

  /**
   * List all known mappings.
   * @returns the collection of mapping names
   */
  static getMappingNames(): Collection

  /**
   * Key iterator over known mapping keys by mapping name.
   * @param mappingName - the mapping name
   * @returns the seekable iterator
   * @throws IllegalArgumentException if mappingName is unknown
   */
  static keyIterator(mappingName: string): SeekableIterator
}
```
