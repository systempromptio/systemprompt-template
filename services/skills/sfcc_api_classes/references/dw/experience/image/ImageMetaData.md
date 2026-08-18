# dw.experience.image.ImageMetaData

## Overview
Holds basic metadata for an image such as width and height (pixels).

## Description
A value object representing image metadata captured when a component attribute referencing an image was stored. Commonly used fields are `width` and `height` (in pixels).

## All Known Subclasses


```ts
declare class ImageMetaData  {
	/**
	 * Image height in pixels (read-only).
	 */
	height: number

	/**
	 * Image width in pixels (read-only).
	 */
	width: number

	/**
	 * Returns image height in pixels.
	 */
	getHeight(): number

	/**
	 * Returns image width in pixels.
	 */
	getWidth(): number
}
```
