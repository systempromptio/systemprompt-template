# dw.customer.oauth.OAuthLoginFlowMgr

## Overview
Utility manager supporting the OAuth Authorization Code Flow with third-party providers: initiate login, obtain tokens and user info, or finalize the full flow.

## Description
Provides static methods to start an OAuth login (returns redirect URL), exchange codes for tokens, retrieve user info for a provider, and finalize the login by performing both steps.

```ts
declare class OAuthLoginFlowMgr {
    /**
     * Completes the OAuth login flow by obtaining tokens and fetching user info.
     * Returns an OAuthFinalizedResponse containing token and user info.
     */
    static finalizeOAuthLogin(): dw.customer.oauth.OAuthFinalizedResponse

    /**
     * Initiates the OAuth login for the configured provider id and returns a URL redirect.
     * @param oauthProviderId provider id as configured in the system
     */
    static initiateOAuthLogin(oauthProviderId: string): dw.web.URLRedirect

    /**
     * Obtains an access token response from the callback request.
     */
    static obtainAccessToken(): dw.customer.oauth.OAuthAccessTokenResponse

    /**
     * Obtains user info for the given provider id and access token.
     * @param oauthProviderId provider id
     * @param accessToken access token string
     */
    static obtainUserInfo(oauthProviderId: string, accessToken: string): dw.customer.oauth.OAuthUserInfoResponse
}
```
