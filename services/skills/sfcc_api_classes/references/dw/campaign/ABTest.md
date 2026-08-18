# dw.campaign.ABTest

## Overview
Represents an AB-test in Commerce Cloud Digital, allowing merchants to compare different storefront experiences by assigning customers to test segments.

## Description
AB-tests enable merchants to test and compare sets of storefront experiences—such as promotions, sorting rules, and slot configurations—by configuring segments and allocation percentages. Tests run for a set period, with customers randomly assigned to segments.

```ts
declare class ABTest extends PersistentObject {
    /**
     * The test ID for this AB-test.
     * @readonly
     */
    readonly ID: string;

    /**
     * Returns the test ID for this AB-test.
     * @returns The test ID for this AB-test.
     */
    getID(): string;
}
```
