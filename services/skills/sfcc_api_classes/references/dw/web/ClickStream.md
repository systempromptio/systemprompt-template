# dw.web.ClickStream

## Overview
Represents the click stream in the session. Records up to 50 clicks per session.

## Description
Represents the click stream in the session. A maximum of 50 clicks is recorded per session. After the maximum is reached, each time the customer clicks on a new link, the oldest click stream entry is purged. The ClickStream always remembers the first click.

The click stream is consulted when retrieving products that the customer has recently visited.

```ts
declare class ClickStream  {
	/**
	 * A collection with all clicks. The first entry is the oldest entry, the last entry is the latest. Returns a copy of the click stream.
	 * @readonly
	 */
	readonly clicks: List

	/**
	 * Identifies if the clickstream recording is enabled or not. It is enabled if Session.isTrackingAllowed() returns true or if that method returns false but the preference 'ClickstreamHonorDNT' is set to false.
	 * @readonly
	 */
	readonly enabled: boolean

	/**
	 * The first click within this session. This first click is stored independent of whether entries are purged.
	 * @readonly
	 */
	readonly first: ClickStreamEntry

	/**
	 * The last recorded click stream, which is also typically the current click. In rare cases (e.g., RedirectURL pipeline) this is not the current click, but instead the last recorded click.
	 * @readonly
	 */
	readonly last: ClickStreamEntry

	/**
	 * Identifies if this is only a partial click stream. If the maximum number of clicks (50) is recorded, the oldest entry is automatically purged with each additional click.
	 * @readonly
	 */
	readonly partial: boolean

	/**
	 * Returns a collection with all clicks. The first entry is the oldest entry, the last entry is the latest. Returns a copy of the click stream, making it safe to work with while it might be modified.
	 * @returns A collection of ClickStreamEntry instances, sorted chronologically
	 */
	getClicks(): List

	/**
	 * Returns the first click within this session. This first click is stored independent of whether entries are purged.
	 * @returns The first click within this session
	 */
	getFirst(): ClickStreamEntry

	/**
	 * Returns the last recorded click stream, which is also typically the current click. In rare cases (e.g., RedirectURL pipeline) this is not the current click, but instead the last recorded click.
	 * @returns The last recorded click stream
	 */
	getLast(): ClickStreamEntry

	/**
	 * Identifies if the clickstream recording is enabled or not. When clickstream tracking is not enabled, getFirst() still operates as expected but the rest of the clicks are not collected.
	 * @returns Whether clickstream tracking is enabled
	 */
	isEnabled(): boolean

	/**
	 * Identifies if this is only a partial click stream. If the maximum number of clicks (50) is recorded, the oldest entry is automatically purged with each additional click.
	 * @returns True if this click stream is partial, false otherwise
	 */
	isPartial(): boolean
}
```
