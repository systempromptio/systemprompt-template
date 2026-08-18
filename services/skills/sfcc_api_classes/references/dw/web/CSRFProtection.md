# dw.web.CSRFProtection

## Overview
Generates and validates CSRF tokens using synchronizer token pattern. Tokens are tied to user sessions with 60-minute validity.

## Description
Used to generate and validate CSRF tokens. CSRFProtection allows applications to protect themselves against CSRF attacks, using synchronizer tokens, a best practice. Once created, these tokens are tied to a user's session and valid for 60 minutes.

Usage: Adding CSRF token to forms:

```html
<form ... action="<protected location>">
  <input name="foo" value="bar">
  <input name="${dw.web.CSRFProtection.getTokenName()}"
            value="${dw.web.CSRFProtection.generateToken()}">
</form>
```

Then, in scripts call:

```javascript
dw.web.CSRFProtection.validateRequest();
```

```ts
declare class CSRFProtection  {
	/**
	 * The system generated CSRF token name. Currently, this name is not user configurable. Must be used for validateRequest() to work.
	 */
	readonly tokenName: string

	/**
	 * Constructs a new unique CSRF token for this session.
	 */
	static generateToken(): string

	/**
	 * Returns the system generated CSRF token name. Currently, this name is not user configurable. Must be used for validateRequest() to work.
	 */
	static getTokenName(): string

	/**
	 * Verifies that a client request contains a valid CSRF token, and that the token has not expired. Returns true if these conditions are met, and false otherwise.
	 */
	static validateRequest(): boolean
}
```
