'use strict';

import { generateRandomString, generateCodeChallenge } from '/js/services/webauthn-utils.js';

const CLIENT_ID = 'marketplace-admin';
const WEBAUTHN_BASE = '/api/v1/core/oauth/webauthn';
const LOGIN_PATH = '/admin/login';
const DEFAULT_REDIRECT = '/control-center';

export { WEBAUTHN_BASE };

export const buildAuthCredentialPayload = (credential) => ({
  id: credential.id,
  rawId: Array.from(new Uint8Array(credential.rawId)),
  response: {
    authenticatorData: Array.from(new Uint8Array(credential.response.authenticatorData)),
    clientDataJSON: Array.from(new Uint8Array(credential.response.clientDataJSON)),
    signature: Array.from(new Uint8Array(credential.response.signature)),
    userHandle: credential.response.userHandle
      ? Array.from(new Uint8Array(credential.response.userHandle))
      : null,
  },
  type: credential.type,
});

export const buildCreationCredentialPayload = (credential) => ({
  id: credential.id,
  rawId: Array.from(new Uint8Array(credential.rawId)),
  response: {
    attestationObject: Array.from(new Uint8Array(credential.response.attestationObject)),
    clientDataJSON: Array.from(new Uint8Array(credential.response.clientDataJSON)),
  },
  type: credential.type,
});

export async function initPkceAndRedirect(userId, authToken, showLoading) {
  const codeVerifier = generateRandomString(64);
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  const csrfState = generateRandomString(32);
  localStorage.setItem('pkce_code_verifier', codeVerifier);
  localStorage.setItem('pkce_csrf_state', csrfState);
  localStorage.setItem('login_redirect', DEFAULT_REDIRECT);
  showLoading('Redirecting...');
  window.location.href = WEBAUTHN_BASE + '/complete?' + new URLSearchParams({
    user_id: userId, auth_token: authToken,
    response_type: 'code', client_id: CLIENT_ID,
    redirect_uri: window.location.origin + LOGIN_PATH, scope: 'user', state: csrfState,
    code_challenge: codeChallenge, code_challenge_method: 'S256',
  }).toString();
}
