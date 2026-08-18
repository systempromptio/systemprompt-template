# dw.customer.oauth.OAuthAccessTokenResponse

## Overview
Holds OAuth token response data returned by a third-party OAuth server when requesting an access token.

## Description
Contains access token, refresh token, expiry, provider id, ID token and any additional tokens returned by the token endpoint.

```ts
declare class OAuthAccessTokenResponse  {
    /** Read-only access token value or null. */
    accessToken: string

    /** Read-only access token expiry (seconds) or null. */
    accessTokenExpiry: number

    /** Read-only error status message or null. */
    errorStatus: string

    /** Read-only map of additional tokens returned in the response. */
    extraTokens: dw.util.Map

    /** Read-only ID token string, if available. */
    IDToken: string

    /** Read-only OAuth provider id. */
    oauthProviderId: string

    /** Read-only refresh token string or null. */
    refreshToken: string

    /** Returns the access token or null. */
    getAccessToken(): string

    /** Returns the access token expiration. */
    getAccessTokenExpiry(): number

    /** Returns the error status or null. */
    getErrorStatus(): string

    /** Returns additional tokens map (may be null or empty). */
    getExtraTokens(): dw.util.Map

    /** Returns the ID token or null. */
    getIDToken(): string

    /** Returns the OAuth provider id. */
    getOauthProviderId(): string

    /** Returns the refresh token or null. */
    getRefreshToken(): string
}
```
