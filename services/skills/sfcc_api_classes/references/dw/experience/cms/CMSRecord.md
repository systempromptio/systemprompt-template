# dw.experience.cms.CMSRecord

## Overview
Represents a Salesforce CMS record exposing `id`, `type`, and `attributes` (typed attribute values resolved to DWScript API objects).

## Description
Encapsulates a CMS record's identifier, content type metadata, and attributes. `attributes` is a `Map` of attribute id → resolved DWScript API object; attributes are also accessible as named properties (e.g. `record.foo`). The `type` value follows the CMS content type schema and includes `id`, `name`, and `attribute_definitions`.

## All Known Subclasses


```ts
declare class CMSRecord  {
	/**
	 * Map of attribute id -> resolved DWScript API object (read-only).
	 */
	attributes: Map

	/**
	 * The record ID (read-only).
	 */
	ID: string

	/**
	 * Type metadata of the CMS record (read-only). Contains `id`, `name`, and `attribute_definitions`.
	 */
	type: Map

	/**
	 * Returns the CMS record attributes as a Map.
	 */
	getAttributes(): Map

	/**
	 * Returns the record ID.
	 */
	getID(): string

	/**
	 * Returns the type metadata for this CMS record.
	 */
	getType(): Map
}
```
