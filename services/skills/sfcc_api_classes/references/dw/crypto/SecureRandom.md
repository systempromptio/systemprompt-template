# dw.crypto.SecureRandom

## Overview
Cryptographically strong random number generator (RNG) adapter producing `Bytes` and numeric values.

## Description
Provides secure randomness for cryptographic use. Offers methods to generate seed bytes, random bytes of a given length, random integers (optionally bounded), and reseed the generator. Typical usage produces a `Bytes` of requested length or nextInt/nextNumber values.

```ts
declare class SecureRandom  {
    /** Instantiate a new SecureRandom */
    constructor()

    /** Returns numBytes of seed material computed by the generator's seed algorithm. */
    generateSeed(numBytes: number): import('../../dw/util/Bytes').Bytes

    /** Generates numBits random bits and returns them as Bytes (length determined by bits). */
    nextBytes(numBits: number): import('../../dw/util/Bytes').Bytes

    /** Returns next pseudorandom int value. */
    nextInt(): number

    /** Returns next pseudorandom int between 0 (inclusive) and upperBound (exclusive). */
    nextInt(upperBound: number): number

    /** Returns next pseudorandom Number between 0.0 (inclusive) and 1.0 (exclusive). */
    nextNumber(): number

    /** Reseeds the generator with additional entropy bytes. */
    setSeed(seed: import('../../dw/util/Bytes').Bytes): void
}
```
