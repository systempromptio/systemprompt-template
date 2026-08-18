# dw.experience.image.Image

## Overview
Represents an image with optional focal point and associated meta data from the site's media library.

## Description
Provides access to the underlying `MediaFile`, an optional `FocalPoint`, and stored `ImageMetaData`. The `metaData` is captured when the component attribute is saved (not refetched on every call). Use `getFile`, `getFocalPoint`, and `getMetaData` to access those values.

## All Known Subclasses


```ts
declare class Image  {
	/**
	 * The image `MediaFile` from the current site's library (read-only).
	 */
	file: MediaFile

	/**
	 * The focal point for the image (read-only).
	 */
	focalPoint: FocalPoint

	/**
	 * Meta data of the physical image file (width/height), obtained at store time (read-only).
	 */
	metaData: ImageMetaData

	/**
	 * Returns the image `MediaFile`, or `null` if not found.
	 */
	getFile(): MediaFile

	/**
	 * Returns the image focal point, or `null` if not provided.
	 */
	getFocalPoint(): FocalPoint

	/**
	 * Returns the stored image meta data, or `null` if none.
	 */
	getMetaData(): ImageMetaData
}
```
