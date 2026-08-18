# dw.system.Session

## Overview
Represents a B2C Commerce session with authentication, custom attributes, click stream tracking, and source code handling.

## Description
Represents a session in B2C Commerce with well-defined attributes like authenticated customer or click stream, plus custom values storage. Session created on first user click (guaranteed unless CDN caching). Uses session stickiness routing requests to same app server. Supports persistent storage for fail-over. Soft timeout at 30 minutes (clears privacy data, allows reopen); hard timeout at 6 hours (invalidates session ID).

Supported session data types: primitives (boolean, number, string, Number, String, Boolean, Date), B2C Commerce value types (Money, Quantity, Decimal, Calendar). Strings limited to 2000 characters. Overall serialized session limited to 10 KB. Unsupported types throw exception in compatibility mode 19.10+.

```ts
declare class Session  {
	/**
	 * Current click stream if this is an HTTP session, null otherwise.
	 * 
	 */
	readonly clickStream: ClickStream

	/**
	 * Currency associated with the current session, established at construction time. Typically equal to site default currency, but may differ in multi-currency sites.
	 */
	currency: Currency

	/**
	 * Session's custom attributes stored for session lifetime, not cleared when customer logs out.
	 * 
	 */
	readonly custom: CustomAttributes

	/**
	 * Customer associated with this storefront session. Always null for non-storefront sessions (jobs, Business Manager). For storefront sessions always returns a customer (may be anonymous if unidentified via cookie).
	 * 
	 */
	readonly customer: Customer

	/**
	 * Whether the customer associated with this session is authenticated. Equivalent to customer.isAuthenticated().
	 * 
	 */
	readonly customerAuthenticated: boolean

	/**
	 * Whether the customer associated with this session is externally authenticated.
	 * 
	 */
	readonly customerExternallyAuthenticated: boolean

	/**
	 * Forms object providing access to all current forms of a customer in the session.
	 * 
	 */
	readonly forms: Forms

	/**
	 * Information on the last source code handled by the session. May or may not be the session's active source code (e.g., last received was inactive and not set as active).
	 * 
	 */
	readonly lastReceivedSourceCodeInfo: SourceCodeInfo

	/**
	 * Session's custom privacy attributes stored for session lifetime, automatically cleared when customer logs out.
	 * 
	 */
	readonly privacy: CustomAttributes

	/**
	 * Unique session ID that can safely be used as identifier against external systems.
	 * 
	 */
	readonly sessionID: String

	/**
	 * Information on the session's active source-code.
	 * 
	 */
	readonly sourceCodeInfo: SourceCodeInfo

	/**
	 * Whether tracking allowed flag is set in the session. Defaults to Site Preference "TrackingAllowed" for new sessions unless "dw_dnt" cookie is found (in which case cookie value takes precedence).
	 */
	trackingAllowed: boolean

	/**
	 * Whether the agent user associated with this session is authenticated.
	 * 
	 */
	readonly userAuthenticated: boolean

	/**
	 * Current agent user name associated with this session. Note: allows access to sensitive security-related data. Pay special attention to PCI DSS v3 requirements 2, 4, and 12.
	 * 
	 */
	readonly userName: String

	/**
	 * Generates a new guest session signature for guest authentication with Shopper Login and API Access Service (SLAS).
	 */
	generateGuestSessionSignature(): String

	/**
	 * Generates a new registered session signature for registered session-bridge call of Shopper Login and API Access Service (SLAS).
	 */
	generateRegisteredSessionSignature(): String

	/**
	 * Returns the current click stream if this is an HTTP session, null otherwise.
	 */
	getClickStream(): ClickStream

	/**
	 * Get the currency associated with the current session. Established at session construction time, typically equal to site default currency, may differ in multi-currency sites.
	 */
	getCurrency(): Currency

	/**
	 * Returns the session's custom attributes stored for session lifetime, not cleared when customer logs out.
	 */
	getCustom(): CustomAttributes

	/**
	 * Returns the customer associated with this storefront session. Always null for non-storefront sessions. For storefront sessions always returns a customer (may be anonymous if unidentified).
	 */
	getCustomer(): Customer

	/**
	 * Returns the forms object that provides access to all current forms of a customer in the session.
	 */
	getForms(): Forms

	/**
	 * Returns information on the last source code handled by the session. May or may not be the session's active source code.
	 */
	getLastReceivedSourceCodeInfo(): SourceCodeInfo

	/**
	 * Returns the session's custom privacy attributes stored for session lifetime, automatically cleared when customer logs out.
	 */
	getPrivacy(): CustomAttributes

	/**
	 * Returns the unique session ID that can safely be used as identifier against external systems.
	 */
	getSessionID(): String

	/**
	 * Returns information on the session's active source-code.
	 */
	getSourceCodeInfo(): SourceCodeInfo

	/**
	 * Returns the current agent user name associated with this session.
	 */
	getUserName(): String

	/**
	 * Identifies whether the customer associated with this session is authenticated. Equivalent to customer.isAuthenticated().
	 */
	isCustomerAuthenticated(): boolean

	/**
	 * Identifies whether the customer associated with this session is externally authenticated.
	 */
	isCustomerExternallyAuthenticated(): boolean

	/**
	 * Returns whether the tracking allowed flag is set in the session.
	 */
	isTrackingAllowed(): boolean

	/**
	 * Identifies whether the agent user associated with this session is authenticated.
	 */
	isUserAuthenticated(): boolean

	/**
	 * Sets the session currency.
	 * @param newCurrency - The new currency to use. Must not be null. Throws exception if currency not allowed by current site.
	 */
	setCurrency(newCurrency: Currency): void

	/**
	 * Applies the specified source code to the current session and basket. Processes exactly as if supplied on URL query string, with benefit of returning error information. If no parameter passed, active source code is removed. May open and commit transaction if none currently active.
	 * @param sourceCode - The source code to set as active in session and basket. If null, active source code is removed.
	 * @returns OK status if source code applied, otherwise ERROR status with possible codes: CODE_INVALID, CODE_INACTIVE.
	 */
	setSourceCode(sourceCode: String): Status

	/**
	 * Sets the tracking allowed flag for the session. If tracking not allowed, multiple services are restricted/disabled: Predictive Intelligence recommendations, Active Data, Analytics of customer behavior. Collected clicks in session click stream are cleared. Setting this property sets session-scoped cookie "dw_dnt" (1=DoNotTrack; 0=Track).
	 * @param trackingAllowed - True if tracking is allowed, false otherwise.
	 */
	setTrackingAllowed(trackingAllowed: boolean): void
}
```
