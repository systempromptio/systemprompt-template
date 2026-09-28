import { rawResponse, errorMessage } from '/js/services/api.js';
import { generateRandomString, generateCodeChallenge } from '/js/services/webauthn-utils.js?v=3';
import {
  DEFAULT_REDIRECT, resolveRedirect, exchangeToken, storeSession
} from '/js/services/webauthn-session.js';
import { safeStorageRemove } from '/js/utils/storage-safe.js';

const CLIENT_ID = 'marketplace-admin';
const LOGIN_PATH = '/admin/login';
const REGISTERED_REDIRECT_URI = LOGIN_PATH;

const form = document.getElementById('odoo-login-form');
const submitBtn = document.getElementById('odoo-sign-in-btn');
const errorBox = document.getElementById('error');
const loading = document.getElementById('loading');
const loadingText = document.getElementById('loading-text');

let inFlight = false;

const showError = (message) => {
  errorBox.textContent = message;
  errorBox.hidden = false;
};

const clearError = () => {
  errorBox.textContent = '';
  errorBox.hidden = true;
};

const setBusy = (message) => {
  loadingText.textContent = message;
  loading.hidden = false;
  form.hidden = true;
};

const clearBusy = () => {
  loading.hidden = true;
  form.hidden = false;
};

const signIn = async (event) => {
  event.preventDefault();
  if (inFlight) return;

  const login = document.getElementById('odoo-login').value.trim();
  const credential = document.getElementById('odoo-credential').value;
  if (!login || !credential) {
    showError('Enter your email and your password or API key.');
    return;
  }

  inFlight = true;
  submitBtn.disabled = true;
  clearError();
  setBusy('Signing in…');

  try {
    const params = new URLSearchParams(window.location.search);

    const thirdPartyClientAwaitingItsOwnCode =
      params.get('client_id') && params.get('redirect_uri');

    if (thirdPartyClientAwaitingItsOwnCode) {
      const body = {
        login,
        credential,
        client_id: params.get('client_id'),
        redirect_uri: params.get('redirect_uri'),
        code_challenge: params.get('code_challenge') || '',
        code_challenge_method: params.get('code_challenge_method') || '',
      };
      for (const key of ['scope', 'state', 'resource']) {
        if (params.get(key)) body[key] = params.get(key);
      }

      const response = await rawResponse('/admin/auth/odoo/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'same-origin',
        body: JSON.stringify(body),
      });
      if (!response.ok) {
        throw new Error(await errorMessage(response) || 'Sign-in failed');
      }

      const { authorization_code: code, redirect_uri: redirectUri, state, issuer } = await response.json();
      if (!code) throw new Error('Sign-in failed — no authorization code was issued.');

      setBusy('Returning to the app…');
      const target = new URL(redirectUri, window.location.origin);
      target.searchParams.set('code', code);
      if (state) target.searchParams.set('state', state);
      if (issuer) target.searchParams.set('iss', issuer);
      window.location.href = target.toString();
      return;
    }

    const codeVerifier = generateRandomString(64);
    const codeChallenge = await generateCodeChallenge(codeVerifier);
    const redirectAfterLogin = params.get('redirect') || DEFAULT_REDIRECT;

    const response = await rawResponse('/admin/auth/odoo/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'same-origin',
      body: JSON.stringify({
        login,
        credential,
        client_id: CLIENT_ID,
        redirect_uri: REGISTERED_REDIRECT_URI,
        code_challenge: codeChallenge,
        code_challenge_method: 'S256',
        state: generateRandomString(32),
      }),
    });

    if (!response.ok) {
      throw new Error(await errorMessage(response) || 'Sign-in failed');
    }

    const { authorization_code: code } = await response.json();
    if (!code) throw new Error('Sign-in failed — no authorization code was issued.');

    setBusy('Finishing…');
    const tokenData = await exchangeToken(code, codeVerifier, REGISTERED_REDIRECT_URI);
    await storeSession(tokenData);
    safeStorageRemove('pkce_code_verifier');
    window.location.href = await resolveRedirect(redirectAfterLogin);
  } catch (err) {
    clearBusy();
    showError(err.message);
  } finally {
    inFlight = false;
    submitBtn.disabled = false;
  }
};

form.addEventListener('submit', signIn);
clearError();
