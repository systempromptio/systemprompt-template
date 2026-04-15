'use strict';

import {
  generateRandomString, generateCodeChallenge,
  preparePublicKeyCredentialRequestOptions, makeRequest
} from '/js/services/webauthn-utils.js';
import { buildAuthCredentialPayload } from '/js/services/webauthn-passkey-helpers.js';
import { showLoading } from '/js/services/webauthn-login-ui.js';

const CLIENT_ID = 'marketplace-admin';
const WEBAUTHN_BASE = '/api/v1/core/oauth/webauthn';
const LOGIN_PATH = '/admin/login';
const DEFAULT_REDIRECT = '/control-center';

export const startPasskeyAuth = async (email) => {
  showLoading('Authenticating...');
  const startResponse = await makeRequest(WEBAUTHN_BASE + '/auth/start?email=' + encodeURIComponent(email), 'POST');
  const publicKeyOptions = preparePublicKeyCredentialRequestOptions(startResponse.data.publicKey);
  const credential = await navigator.credentials.get({ publicKey: publicKeyOptions });
  if (!credential) throw new Error('Authentication was cancelled');
  return { startResponse, credential };
};

export const finishPasskeyAuth = async (startResponse, credential) => {
  showLoading('Verifying...');
  return makeRequest(WEBAUTHN_BASE + '/auth/finish', 'POST', {
    challenge_id: startResponse.data.challenge_id,
    credential: buildAuthCredentialPayload(credential),
  });
};

export const redirectWithPkce = async (finishResponse) => {
  const { user_id: userId, auth_token: authToken } = finishResponse.data || {};
  if (!userId || typeof authToken !== 'string' || authToken.length === 0) {
    throw new Error('Login session invalid — please reload this page and try again.');
  }
  const codeVerifier = generateRandomString(64);
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  const csrfState = generateRandomString(32);
  const params = new URLSearchParams(window.location.search);
  localStorage.setItem('pkce_code_verifier', codeVerifier);
  localStorage.setItem('pkce_csrf_state', csrfState);
  localStorage.setItem('login_redirect', params.get('redirect') || DEFAULT_REDIRECT);
  showLoading('Redirecting...');
  window.location.href = WEBAUTHN_BASE + '/complete?' + new URLSearchParams({
    user_id: userId, auth_token: authToken,
    response_type: 'code', client_id: CLIENT_ID,
    redirect_uri: window.location.origin + LOGIN_PATH, scope: 'user', state: csrfState,
    code_challenge: codeChallenge, code_challenge_method: 'S256',
  }).toString();
};
