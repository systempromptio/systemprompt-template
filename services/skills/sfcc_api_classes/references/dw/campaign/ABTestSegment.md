# dw.campaign.ABTestSegment

## Overview
Represents an AB-test segment in Commerce Cloud Digital. Each AB-test defines one or more segments to which customers are randomly assigned.

## Description
Each segment defines a set of experiences for customers in that segment. There is always one control segment with the default experiences for the site.

```ts
declare class ABTestSegment extends PersistentObject {
    /**
     * Get the AB-test to which this segment belongs.
     */
    getABTest(): ABTest;

    /**
     * Get the ID of the AB-test segment.
     */
    getID(): String;

    /**
     * Returns true if this is the control segment for the AB-test (no experiences associated).
     */
    isControlSegment(): boolean;

    /**
     * The AB-test to which this segment belongs.
     * @readonly
     */
    readonly ABTest: ABTest;

    /**
     * Returns true if this is the control segment for the AB-test.
     * @readonly
     */
    readonly controlSegment: boolean;

    /**
     * The ID of the AB-test segment.
     * @readonly
     */
    readonly ID: String;
}
```
