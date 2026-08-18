# dw.campaign.ABTestMgr

## Overview
Manager class to access AB-test information in the storefront.

## Description
Provides access to AB-test segments and allows checking if the current customer is a participant in a specific segment.

```ts
declare class ABTestMgr {
    /**
     * Returns the AB-test segments to which the current customer is assigned.
     * AB-test segments deleted in the meantime will not be returned.
     */
    static getAssignedTestSegments(): Collection;

    /**
     * Test whether the current customer is a member of the specified AB-test segment.
     * @param testID The ID of the AB-test, must not be null.
     * @param segmentID The ID of the segment within the AB-test, must not be null.
     * @returns True if the current customer is a member of the specified segment, false otherwise.
     */
    static isParticipant(testID: String, segmentID: String): boolean;

    /**
     * Returns the AB-test segments to which the current customer is assigned.
     * @readonly
     */
    static readonly assignedTestSegments: Collection;
}
```
