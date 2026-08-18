# dw.system.Status

## Overview
Bundles one or more StatusItems into an API result descriptor that reports an overall OK or ERROR state.

## Description
The first ERROR StatusItem (or the first StatusItem when there are no errors) determines the reported code, message, details, and parameters; clients should use getCode() as the permanent identifier because the message text can change. Helper methods mirror that priority when adding details or items.

```ts
declare class Status  {
    /**
     * Numeric code that signals an ERROR overall.
     */
    static ERROR: number

    /**
     * Numeric code that signals an OK overall.
     */
    static OK: number

    /**
     * The status code from the first ERROR StatusItem or the first item when no errors exist.
     */
    code: string

    /**
     * Details from the first ERROR StatusItem or, if none, the first StatusItem.
     */
    readonly details: Map<string, Object>

    /**
     * True when the status contains at least one ERROR StatusItem.
     */
    readonly error: boolean

    /**
     * All status items bundled into this Status.
     */
    readonly items: List

    /**
     * Message from the first ERROR StatusItem or the first StatusItem when there is no error.
     */
    readonly message: string



    /**
     * Parameters from the first ERROR StatusItem or, if none, the first StatusItem.
     */
    readonly parameters: List

    /**
    
     * Either OK or ERROR depending on the contained StatusItems.
     */
    readonly status: number

    /**
     * Adds detail information for the first ERROR StatusItem or the first StatusItem when no errors exist.
     */
    addDetail(key: string, value: Object): void

    /**
     * Appends another StatusItem to this Status.
     */
    addItem(item: StatusItem): void

    /**
     * Returns the code from the first ERROR StatusItem or the first StatusItem when no errors exist.
     */
    getCode(): string

    /**
     * Returns the detail value for the supplied key on the prioritized StatusItem.
     */
    getDetail(key: string): Object

    /**
     * Returns the detail map for the prioritized StatusItem.
     */
    getDetails(): Map

    /**
     * Returns every StatusItem in this Status.
     */
    getItems(): List

    /**
     * Returns the message from the prioritized StatusItem.
     */
    getMessage(): string

    /**
     * Returns the parameters from the prioritized StatusItem.
     */
    getParameters(): List

    /**
     * Returns either OK or ERROR based on the contained StatusItems.
     */
    getStatus(): number

    /**
     * Returns true when at least one StatusItem is an ERROR.
     */
    isError(): boolean
}
```