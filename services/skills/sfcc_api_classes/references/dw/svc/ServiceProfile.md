# dw.svc.ServiceProfile

## Overview
Configuration object for service profiles including circuit breaker, rate limiter, and timeout settings.

## Description
Stores configuration for Service Profiles including circuit breaker limits, rate limiting, and timeout values. All properties are read-only and managed via Business Manager.

```ts
declare class ServiceProfile extends ExtensibleObject {
	/**
	 * Maximum number of errors in an interval allowed by the circuit breaker.
	 * @readonly
	 */
	readonly cbCalls: number
	
	/**
	 * Interval of the circuit breaker in milliseconds.
	 * @readonly
	 */
	readonly cbMillis: number
	
	/**
	 * Unique Service ID.
	 * @readonly
	 */
	readonly ID: string
	
	/**
	 * Maximum number of calls in an interval allowed by the rate limiter.
	 * @readonly
	 */
	readonly rateLimitCalls: number
	
	/**
	 * Interval of the rate limiter in milliseconds.
	 * @readonly
	 */
	readonly rateLimitMillis: number
	
	/**
	 * Service call timeout in milliseconds.
	 * @readonly
	 */
	readonly timeoutMillis: number
	
	/**
	 * Returns the maximum number of errors in an interval allowed by the circuit breaker.
	 * @returns Maximum number of errors in an interval allowed by the circuit breaker
	 */
	getCbCalls(): number
	
	/**
	 * Returns the interval of the circuit breaker in milliseconds.
	 * @returns Circuit breaker interval in milliseconds
	 */
	getCbMillis(): number
	
	/**
	 * Returns the unique Service ID.
	 * @returns unique Service ID
	 */
	getID(): string
	
	/**
	 * Returns the maximum number of calls in an interval allowed by the rate limiter.
	 * @returns Maximum number of calls in an interval allowed by the rate limiter
	 */
	getRateLimitCalls(): number
	
	/**
	 * Returns the interval of the rate limiter in milliseconds.
	 * @returns Interval of the rate limiter in milliseconds
	 */
	getRateLimitMillis(): number
	
	/**
	 * Returns the service call timeout in milliseconds.
	 * @returns Service call timeout in milliseconds
	 */
	getTimeoutMillis(): number
}
```
