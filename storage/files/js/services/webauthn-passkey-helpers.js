import {
  generateRandomString, generateCodeChallenge, arrayBufferToBase64url
} from '/js/services/webauthn-utils.js?v=3';

const CLIENT_ID = 'marketplace-admin';
const WEBAUTHN_BASE = '/api/v1/core/oauth/webauthn';
const LOGIN_PATH = '/admin/login';
const DEFAULT_REDIRECT = '/admin/profile';

export { WEBAUTHN_BASE };

export const buildAuthCredentialPayload = (credential) => ({
  id: credential.id,
  rawId: arrayBufferToBase64url(credential.rawId),
  response: {
    authenticatorData: arrayBufferToBase64url(credential.response.authenticatorData),
    clientDataJSON: arrayBufferToBase64url(credential.response.clientDataJSON),
    signature: arrayBufferToBase64url(credential.response.signature),
    userHandle: credential.response.userHandle
      ? arrayBufferToBase64url(credential.response.userHandle)
      : null,
  },
  type: credential.type,
});

export const buildCreationCredentialPayload = (credential) => ({
  id: credential.id,
  rawId: arrayBufferToBase64url(credential.rawId),
  response: {
    attestationObject: arrayBufferToBase64url(credential.response.attestationObject),
    clientDataJSON: arrayBufferToBase64url(credential.response.clientDataJSON),
  },
  type: credential.type,
});

export async function initPkceAndRedirect(userId, authToken, showLoading) {
  if (!userId || typeof authToken !== 'string' || authToken.length === 0) {
    throw new Error('Login session invalid — please reload this page and try again.');
  }
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
