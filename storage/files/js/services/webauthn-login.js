'use strict';

import {
  getEmailInput, clearAccessToken, showError, showLoginForm,
  showLoading, showPasskeyError, showEmailError,
  hasValidAdminToken, initMagicLinkUI
} from '/js/services/webauthn-login-ui.js';
import { startPasskeyAuth, finishPasskeyAuth, redirectWithPkce } from '/js/services/webauthn-helpers.js';
import { showToast } from '/js/services/toast.js';

const CLIENT_ID = 'marketplace-admin';
const OAUTH_BASE = '/api/v1/core/oauth';
const LOGIN_PATH = '/admin/login';
const DEFAULT_REDIRECT = '/control-center';
const signInBtn = document.getElementById('sign-in-btn');
const emailInput = getEmailInput();
let isAuthenticating = false;

const exchangeToken = async (code, codeVerifier) => {
  const tokenResponse = await fetch(OAUTH_BASE + '/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    credentials: 'same-origin',
    body: new URLSearchParams({
      grant_type: 'authorization_code', code,
      redirect_uri: window.location.origin + LOGIN_PATH,
      code_verifier: codeVerifier, client_id: CLIENT_ID,
    }),
  });
  const tokenData = await tokenResponse.json();
  if (!tokenResponse.ok) throw new Error(tokenData.error_description || tokenData.error || 'Token exchange failed');
  return tokenData;
};

const storeSession = async (tokenData) => {
  const response = await fetch('/api/public/auth/session', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'same-origin',
    body: JSON.stringify({ access_token: tokenData.access_token, expires_in: tokenData.expires_in || 3600 }),
  });
  if (!response.ok) {
    const data = await response.json().catch(() => ({}));
    throw new Error(data.message || data.error || 'Failed to store session');
  }
  if (tokenData.refresh_token) localStorage.setItem('refresh_token', tokenData.refresh_token);
};

const completePendingRegistration = async () => {
  const pendingReg = localStorage.getItem('pending_registration');
  if (pendingReg) {
    try {
      const response = await fetch('/api/public/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'same-origin',
        body: pendingReg,
      });
      if (!response.ok) {
        const data = await response.json().catch(() => ({}));
        showToast(data.message || data.error || 'Registration could not be completed.', 'error');
      }
    } catch (_err) {
      showToast('Registration could not be completed. Please try again.', 'error');
    }
    localStorage.removeItem('pending_registration');
  }
};

const processCallback = async (code) => {
  const codeVerifier = localStorage.getItem('pkce_code_verifier');
  const redirectAfterLogin = localStorage.getItem('login_redirect') || DEFAULT_REDIRECT;
  if (!codeVerifier) {
    window.history.replaceState({}, '', LOGIN_PATH);
    showLoginForm();
  } else {
    try {
      showLoading('Exchanging token...');
      const tokenData = await exchangeToken(code, codeVerifier);
      await storeSession(tokenData);
      await completePendingRegistration();
      localStorage.removeItem('pkce_code_verifier');
      localStorage.removeItem('pkce_csrf_state');
      localStorage.removeItem('login_redirect');
      window.location.href = redirectAfterLogin;
    } catch (err) {
      showError(err.message);
      window.history.replaceState({}, '', LOGIN_PATH);
    }
  }
};

const handleCallback = async () => {
  const params = new URLSearchParams(window.location.search);
  const error = params.get('error');
  if (error) {
    showError(params.get('error_description') || error);
    return true;
  }
  const code = params.get('code');
  if (code) {
    await processCallback(code);
    return true;
  }
  return false;
};

const authenticateWithPasskey = async () => {
  if (isAuthenticating) return;
  const email = emailInput.value.trim();
  if (!email) {
    showEmailError('Please enter your email address.');
  } else {
    isAuthenticating = true;
    signInBtn.disabled = true;
    try {
      const { startResponse, credential } = await startPasskeyAuth(email);
      const finishResponse = await finishPasskeyAuth(startResponse, credential);
      await redirectWithPkce(finishResponse);
    } catch (error) {
      showPasskeyError(error);
    } finally {
      isAuthenticating = false;
      signInBtn.disabled = false;
    }
  }
};

signInBtn.addEventListener('click', authenticateWithPasskey);
emailInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') { e.preventDefault(); authenticateWithPasskey(); }
});
initMagicLinkUI();

(async () => {
  const wasCallback = await handleCallback();
  if (!wasCallback) {
    if (hasValidAdminToken()) {
      const params = new URLSearchParams(window.location.search);
      window.location.href = params.get('redirect') || DEFAULT_REDIRECT;
    } else {
      await clearAccessToken();
      if (!window.PublicKeyCredential) {
        showError('Your browser does not support passkeys. Please use a modern browser (Chrome, Firefox, Safari, Edge).');
      } else {
        showLoginForm();
      }
    }
  }
})();
